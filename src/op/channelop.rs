use std::future::Future;
use std::task::{Context, Poll};

use tokio::sync::mpsc;

use super::*;

/// Create a connected sending and receiving channel pair, [`ChannelPushOp`] and [`ChannelPullOp`].
///
/// Only supports owned [`Df`] flows.
pub fn channel_op<T>(buffer: usize) -> (ChannelPushOp<T>, ChannelPullOp<T>) {
    let (send, recv) = mpsc::channel(buffer);
    (ChannelPushOp::new(send), ChannelPullOp::new(recv))
}

/// The sending (push) half of a channel.
pub struct ChannelPushOp<T> {
    send: mpsc::Sender<T>,
}
impl<T> ChannelPushOp<T> {
    /// Wraps a [`tokio::sync::mpsc::Sender`] to create a new ChannelPushOp.
    /// Note the [`channel_op`] function should be prefered over this constructor.
    pub fn new(send: mpsc::Sender<T>) -> Self {
        Self { send }
    }
}
impl<T> Op for ChannelPushOp<T> {}
impl<T> PushOp for ChannelPushOp<T> {
    type Inflow = Df<T>;
}
impl<T> MovePushOp for ChannelPushOp<T> {
    type Feedback = impl Future;

    #[must_use]
    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        let send = self.send.clone();
        async move { send.clone().send(item).await }
    }
}

/// The receiving (pull) half of a channel.
///
/// Supports only owned [`Df`] flows.
pub struct ChannelPullOp<T> {
    recv: mpsc::Receiver<T>,
}
impl<T> ChannelPullOp<T> {
    /// Wraps a [`tokio::sync::mpsc::Receiver`] to create a new ChannelPullOp.
    /// Note the [`channel_op`] function should be prefered over this constructor.
    pub fn new(recv: mpsc::Receiver<T>) -> Self {
        Self { recv }
    }
}
impl<T> Op for ChannelPullOp<T> {}
impl<T> PullOp for ChannelPullOp<T> {
    type Outflow = Df<T>;
}
impl<T> MovePullOp for ChannelPullOp<T> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        self.recv.poll_recv(ctx)
    }
}