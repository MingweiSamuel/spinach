use std::task::{Context, Poll};

use crate::flow::*;

use super::*;

/// An Op for converting a ref flow into an owned flow via [`Clone`].
///
/// Supports both [`Df`] and [`Rx`].
///
/// To go from owned to ref (the opposite of this), use [`ReferenceOp`].
pub struct CloneOp<O: Op> {
    op: O,
}
impl<O: Op> CloneOp<O> {
    /// Create a CloneOp from an existing Op. Note the values received
    /// by this op must implement [`Clone`] for this op to be usable.
    pub fn new(op: O) -> Self {
        CloneOp { op }
    }
}
impl<O: Op> Op for CloneOp<O> {}
impl<O: PullOp> PullOp for CloneOp<O> {
    type Outflow = O::Outflow;
}
impl<O: PushOp> PushOp for CloneOp<O> {
    type Inflow = O::Inflow;
}
impl<O: RefPullOp> MovePullOp for CloneOp<O>
where
    <O::Outflow as Flow>::Domain: Clone,
{
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        let polled = self.op.poll_next(ctx);
        polled.map(|opt| opt.cloned())
    }
}
impl<O: MovePushOp> RefPushOp for CloneOp<O>
where
    <O::Inflow as Flow>::Domain: Clone,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback {
        self.op.push(item.clone())
    }
}
