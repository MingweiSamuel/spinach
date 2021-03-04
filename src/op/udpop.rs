use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll};

use tokio::io::ReadBuf;
use tokio::net::UdpSocket;

use crate::flow::*;

use super::*;

/// Buffer size for the [`UdpPullOp`]. NOTE that any packets longer than this will be truncated!
pub const UDP_BUFFER: usize = 4096;

/// Create a pull and push pair from a [`UdpSocket`].
pub fn udp_op<T: AsRef<[u8]>>(sock: UdpSocket) -> (UdpPullOp, UdpPushOp<T>) {
    let sock = Arc::new(sock);
    (UdpPullOp::new(sock.clone()), UdpPushOp::new(sock))
}

/// The receving (pull) side of a UDP connection.
pub struct UdpPullOp {
    sock: Arc<UdpSocket>,
    buffer: [u8; UDP_BUFFER],
}
impl UdpPullOp {
    pub fn new(sock: Arc<UdpSocket>) -> Self {
        Self {
            sock,
            buffer: [0; UDP_BUFFER],
        }
    }
}
impl Op for UdpPullOp {}
impl PullOp for UdpPullOp {
    type Outflow = Df;
    type Outdomain<'s> = &'s [u8];

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
        let mut readbuf = ReadBuf::new(&mut self.buffer);
        match self.sock.poll_recv(ctx, &mut readbuf) {
            Poll::Ready(Ok(())) => {
                let len = readbuf.filled().len();
                Poll::Ready(Some(&self.buffer[..len]))
            }
            Poll::Ready(Err(err)) => {
                println!("{}", err);
                Poll::Ready(None) // ERR => EOS
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// The sending (push) side of a UDP connection.
pub struct UdpPushOp<T: 'static + AsRef<[u8]>> {
    sock: Arc<UdpSocket>,
    _phantom: std::marker::PhantomData<T>,
}
impl<T: 'static + AsRef<[u8]>> UdpPushOp<T> {
    pub fn new(sock: Arc<UdpSocket>) -> Self {
        Self { sock, _phantom: std::marker::PhantomData }
    }
}
impl<T: 'static + AsRef<[u8]>> Op for UdpPushOp<T> {}
impl<T: 'static + AsRef<[u8]>> PushOp for UdpPushOp<T> {
    type Inflow = Df;
    type Indomain<'p> = T;

    type Feedback = impl Future;

    #[must_use]
    fn push<'p>(&mut self, item: Self::Indomain<'p>) -> Self::Feedback {
        let slice = item.as_ref();
        if slice.len() > UDP_BUFFER {
            panic!(
                "Message length {} longer than limit, {}.",
                slice.len(),
                UDP_BUFFER
            );
        }
        let sock = self.sock.clone();
        async move {
            let slice = item.as_ref();
            sock.send(slice).await
        }
    }
}
