use std::future::Future;
use std::task::{Context, Poll};

use tokio::join;

use super::*;

/// An Op which pushes to two downstream ops.
pub struct SplitOp<O: PushOp, P: PushOp<Inflow = O::Inflow, Indomain = O::Indomain>> {
    op0: O,
    op1: P,
}
impl<O: PushOp, P: PushOp<Inflow = O::Inflow, Indomain = O::Indomain>> SplitOp<O, P> {
    /// Split op to op0 and op1.
    pub fn new(op0: O, op1: P) -> Self {
        Self { op0, op1 }
    }
}
impl<O: PushOp, P: PushOp<Inflow = O::Inflow, Indomain = O::Indomain>> Op for SplitOp<O, P> {}
impl<O: PushOp, P: PushOp<Inflow = O::Inflow, Indomain = O::Indomain>> PushOp for SplitOp<O, P> {
    type Inflow = O::Inflow;
    type Indomain = O::Indomain;
}
impl<O: MovePushOp, P: MovePushOp<Inflow = O::Inflow, Indomain = O::Indomain>> MovePushOp
    for SplitOp<O, P>
where
    O::Indomain: Clone,
{
    type Feedback = impl Future;

    fn push(&mut self, item: Self::Indomain) -> Self::Feedback {
        let f0 = self.op0.push(item.clone());
        let f1 = self.op1.push(item);
        async move { join!(f0, f1) }
    }
}
impl<O: RefPushOp, P: RefPushOp<Inflow = O::Inflow, Indomain = O::Indomain>> RefPushOp
    for SplitOp<O, P>
{
    type Feedback = impl Future;

    fn push(&mut self, item: &Self::Indomain) -> Self::Feedback {
        let f0 = self.op0.push(item);
        let f1 = self.op1.push(item);
        async move { join!(f0, f1) }
    }
}

/// An Op which receives from two upstream ops.
///
/// Note that this is biased, it will give priority to the first op, then the second op.
pub struct MergeOp<O: PullOp, P: PullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> {
    op0: O,
    op1: P,
}
impl<O: PullOp, P: PullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> MergeOp<O, P> {
    /// Receive from both OP0 and OP1, where priority is to pull from OP0.
    pub fn new(op0: O, op1: P) -> Self {
        Self { op0, op1 }
    }
}
impl<O: PullOp, P: PullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> Op for MergeOp<O, P> {}
impl<O: PullOp, P: PullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> PullOp
    for MergeOp<O, P>
{
    type Outflow = O::Outflow;
    type Outdomain = O::Outdomain;
}
impl<O: MovePullOp, P: MovePullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> MovePullOp
    for MergeOp<O, P>
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        let poll0 = self.op0.poll_next(ctx);
        if let Poll::Ready(Some(item)) = poll0 {
            return Poll::Ready(Some(item));
        }
        let poll1 = self.op1.poll_next(ctx);
        if let Poll::Ready(Some(item)) = poll1 {
            return Poll::Ready(Some(item));
        }
        if poll0.is_ready() && poll1.is_ready() {
            // Both EOS.
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}
impl<O: RefPullOp, P: RefPullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> RefPullOp
    for MergeOp<O, P>
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Outdomain>> {
        let poll0 = self.op0.poll_next(ctx);
        if let Poll::Ready(Some(item)) = poll0 {
            return Poll::Ready(Some(item));
        }
        let poll1 = self.op1.poll_next(ctx);
        if let Poll::Ready(Some(item)) = poll1 {
            return Poll::Ready(Some(item));
        }
        if poll0.is_ready() && poll1.is_ready() {
            // Both EOS.
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}
