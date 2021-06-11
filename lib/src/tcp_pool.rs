use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Context};
use std::net::SocketAddr;

use bytes::{BufMut, BytesMut};
use futures::sink::SinkExt;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{Result};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_stream::Stream;


struct TcpPoolInternal {
    listener: TcpListener,
    streams: Mutex<HashMap<SocketAddr, Framed<TcpStream, LengthDelimitedCodec>>>,
}

pub struct TcpPool {
    handle: Arc<TcpPoolInternal>,
}

impl Clone for TcpPool {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl TcpPool {
    pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
        let result_listener = TcpListener::bind(addr).await;
        result_listener
            .map(|listener| {
                let handle = Arc::new(TcpPoolInternal {
                    listener,
                    streams: Default::default(),
                });
                Self { handle }
            })
    }

    pub async fn write<T: Clone + Serialize + DeserializeOwned>(
        &self, addr: SocketAddr, item: &T
    )
        -> Result<()>
    {
        let mut writer = BytesMut::new().writer();
        serde_json::to_writer(&mut writer, item)?;
        let bytes = writer.into_inner().freeze();

        {
            let mut streams = self.handle.streams.lock().expect("Poisoned");
            match streams.get_mut(&addr) {
                Some(stream) => {
                    stream.send(bytes).await?;
                    Ok(())
                }
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    format!("Addr not found: {}.", addr)))
            }
        }
    }

    pub fn poll_accept(&self, ctx: &mut Context<'_>) -> Poll<Result<SocketAddr>> {
        match self.handle.listener.poll_accept(ctx) {
            Poll::Ready(Ok((stream, addr))) => {

                let framed_stream = LengthDelimitedCodec::builder()
                    .length_field_length(2)
                    .new_framed(stream);

                {
                    let mut streams = self.handle.streams.lock().expect("Poisoned");
                    streams.insert(addr, framed_stream);
                }

                Poll::Ready(Ok(addr))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }

    pub fn poll_read<T: Clone + Serialize + DeserializeOwned>(
        &self, ctx: &mut Context<'_>
    )
        -> Poll<Option<(SocketAddr, T)>>
    {
        let mut item = None;
        {
            let mut streams = self.handle.streams.lock().expect("Poisoned");

            streams.retain(|addr, stream| {
                if item.is_some() { // "break"
                    return true;
                }

                match Pin::new(stream).poll_next(ctx) {
                    Poll::Ready(Some(Ok(bytes))) => {
                        item.replace((*addr, bytes));
                        true
                    }
                    Poll::Ready(Some(Err(err))) => {
                        eprintln!("TCP ERROR on {}: {}", addr, err);
                        true // TODO?
                    }
                    Poll::Ready(None) => false,
                    Poll::Pending => true,
                }
            });
        }

        match item {
            Some((addr, bytes)) => {
                match serde_json::from_slice(&*bytes) {
                    Ok(item) => Poll::Ready(Some((addr, item))),
                    Err(err) => {
                        eprintln!("SERDE DE ERR !!!1 {:?}", err);
                        Poll::Pending
                    }
                }
            }
            None => Poll::Pending
        }
    }
}