use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Context};
use std::net::SocketAddr;

use bytes::{Bytes, BytesMut};
use futures::sink::SinkExt;
use tokio::io::{Result};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_stream::Stream;


struct TcpServerInternal {
    listener: TcpListener,
    streams: Mutex<HashMap<SocketAddr, Framed<TcpStream, LengthDelimitedCodec>>>,
}

pub struct TcpServer {
    handle: Arc<TcpServerInternal>,
}

impl Clone for TcpServer {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl TcpServer {
    pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
        let result_listener = TcpListener::bind(addr).await;
        result_listener
            .map(|listener| {
                let handle = Arc::new(TcpServerInternal {
                    listener,
                    streams: Default::default(),
                });
                Self { handle }
            })
    }

    pub async fn write(&self, addr: SocketAddr, item: Bytes) -> Result<()> {
        // let mut writer = BytesMut::new().writer();
        // serde_json::to_writer(&mut writer, item)?;
        // let bytes = writer.into_inner().freeze();

        {
            let mut streams = self.handle.streams.lock().expect("Poisoned");
            match streams.get_mut(&addr) {
                Some(stream) => {
                    stream.send(item).await?;
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

    pub fn poll_read(&self, ctx: &mut Context<'_>) -> Poll<Option<(SocketAddr, BytesMut)>> {
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
            Some((addr, bytes)) => Poll::Ready(Some((addr, bytes))),
            None => Poll::Pending,
        }
    }
}