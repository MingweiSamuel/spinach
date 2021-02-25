use std::future::Future;
use std::task::{Context, Poll};

use tokio::join;

use crate::flow::*;

use super::*;

/// An Op which pushes to two downstream ops.
pub struct SplitOp<O: PushOp, P: PushOp<Inflow = O::Inflow>> {
    op0: O,
    op1: P,
}
impl<O: PushOp, P: PushOp<Inflow = O::Inflow>> SplitOp<O, P> {
    /// Split op to op0 and op1.
    pub fn new(op0: O, op1: P) -> Self {
        SplitOp { op0, op1 }
    }
}
impl<O: PushOp, P: PushOp<Inflow = O::Inflow>> Op for SplitOp<O, P> {}
impl<O: PushOp, P: PushOp<Inflow = O::Inflow>> PushOp for SplitOp<O, P> {
    type Inflow = O::Inflow;
}
impl<O: MovePushOp, P: MovePushOp<Inflow = O::Inflow>> MovePushOp for SplitOp<O, P>
where
    <O::Inflow as Flow>::Domain: Clone,
{
    type Feedback = impl Future;

    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        let f0 = self.op0.push(item.clone());
        let f1 = self.op1.push(item);
        async move { join!(f0, f1) }
    }
}
impl<O: RefPushOp, P: RefPushOp<Inflow = O::Inflow>> RefPushOp for SplitOp<O, P> {
    type Feedback = impl Future;

    fn push(&mut self, item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback {
        let f0 = self.op0.push(item);
        let f1 = self.op1.push(item);
        async move { join!(f0, f1) }
    }
}

/// An Op which receives from two upstream ops.
///
/// Note that this is biased, it will give priority to the first op, then the second op.
pub struct MergeOp<O: PullOp, P: PullOp<Outflow = O::Outflow>> {
    op0: O,
    op1: P,
}
impl<O: PullOp, P: PullOp<Outflow = O::Outflow>> MergeOp<O, P> {
    /// Receive from both OP0 and OP1, where priority is to pull from OP0.
    pub fn new(op0: O, op1: P) -> Self {
        MergeOp { op0, op1 }
    }
}
impl<O: PullOp, P: PullOp<Outflow = O::Outflow>> Op for MergeOp<O, P> {}
impl<O: PullOp, P: PullOp<Outflow = O::Outflow>> PullOp for MergeOp<O, P> {
    type Outflow = O::Outflow;
}
impl<O: MovePullOp, P: MovePullOp<Outflow = O::Outflow>> MovePullOp for MergeOp<O, P> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
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
impl<O: RefPullOp, P: RefPullOp<Outflow = O::Outflow>> RefPullOp for MergeOp<O, P> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
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
