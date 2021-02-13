use std::future::Future;
use std::task::{ Context, Poll };

use tokio::sync::mpsc;

use super::op::*;
use super::types::*;



pub fn channel_op<T>(buffer: usize) -> ( ChannelPushOp<T>, ChannelPullOp<T> ) {
    let ( send, recv ) = mpsc::channel(buffer);
    ( ChannelPushOp::new(send), ChannelPullOp::new(recv) )
}



pub struct ChannelPushOp<T> {
    send: mpsc::Sender<T>,
}
impl<T> ChannelPushOp<T> {
    pub fn new(send: mpsc::Sender<T>) -> Self {
        Self {
            send: send,
        }
    }
}
impl<T> Op for ChannelPushOp<T> {}
impl<T> PushOp for ChannelPushOp<T> {
    type Inflow = DF;
    type Domain = T;
}
impl<T> MovePushOp for ChannelPushOp<T> {
    type Feedback = impl Future;

    #[must_use]
    fn push(&mut self, item: Self::Domain) -> Self::Feedback {
        let send = self.send.clone();
        async move {
            send.clone().send(item).await
        }
    }
}



pub struct ChannelPullOp<T> {
    recv: mpsc::Receiver<T>,
}
impl<T> ChannelPullOp<T> {
    pub fn new(recv: mpsc::Receiver<T>) -> Self {
        Self {
            recv: recv,
        }
    }
}
impl<T> Op for ChannelPullOp<T> {}
impl<T> PullOp for ChannelPullOp<T> {
    type Outflow = DF;
    type Codomain = T;
}
impl<T> MovePullOp for ChannelPullOp<T> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>> {
        self.recv.poll_recv(ctx)
    }
}


