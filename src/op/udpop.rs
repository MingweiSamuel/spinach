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
pub fn udp_op(sock: UdpSocket) -> (UdpPullOp, UdpPushOp) {
    let sock = Arc::new(sock);
    (UdpPullOp::new(sock.clone()), UdpPushOp::new(sock))
}

/// The receving (pull) side of a udp connection.
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
    type Outflow = Df<Vec<u8>>;
}
impl MovePullOp for UdpPullOp {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        let mut readbuf = ReadBuf::new(&mut self.buffer);
        match self.sock.poll_recv(ctx, &mut readbuf) {
            Poll::Ready(Ok(())) => Poll::Ready(Some(readbuf.filled().into())),
            Poll::Ready(Err(err)) => {
                println!("{}", err);
                Poll::Ready(None) // ERR => EOS
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// The sending (push) side of a udp connection.
pub struct UdpPushOp {
    sock: Arc<UdpSocket>,
}
impl UdpPushOp {
    pub fn new(sock: Arc<UdpSocket>) -> Self {
        Self { sock }
    }
}
impl Op for UdpPushOp {}
impl PushOp for UdpPushOp {
    type Inflow = Df<Vec<u8>>;
}
impl MovePushOp for UdpPushOp {
    type Feedback = impl Future;

    #[must_use]
    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        if item.len() > UDP_BUFFER {
            panic!(
                "Message length {} longer than limit, {}.",
                item.len(),
                UDP_BUFFER
            );
        }
        let sock = self.sock.clone();
        async move { sock.send(&item[..]).await }
    }
}
