use std::fmt::Debug;
use std::task::{Context, Poll};

use crate::flow::*;

use super::*;

/// An Op which logs each passing element to stdout, for debugging.
///
/// Supports both owned values and refs, and [`Df`] and [`Rx`] pipes.
pub struct DebugOp<O: Op> {
    op: O,
    tag: &'static str,
}
impl<O: Op> DebugOp<O> {
    /// Wrap OP, log with the tag TAG.
    pub fn new(op: O, tag: &'static str) -> Self {
        Self { op, tag }
    }
}
impl<O: Op> Op for DebugOp<O> {}
impl<O: PullOp> PullOp for DebugOp<O> {
    type Outflow = O::Outflow;
}
impl<O: PushOp> PushOp for DebugOp<O> {
    type Inflow = O::Inflow;
}
impl<O: MovePullOp> MovePullOp for DebugOp<O>
where
    <O::Outflow as Flow>::Domain: Debug,
{
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        let polled = self.op.poll_next(ctx);
        if let Poll::Ready(Some(item)) = &polled {
            println!("{}: {:?}", self.tag, item);
        }
        polled
    }
}
impl<O: RefPullOp> RefPullOp for DebugOp<O>
where
    <O::Outflow as Flow>::Domain: Debug,
{
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
        let polled = self.op.poll_next(ctx);
        if let Poll::Ready(Some(item)) = &polled {
            println!("{}: {:?}", self.tag, item);
        }
        polled
    }
}
impl<O: MovePushOp> MovePushOp for DebugOp<O>
where
    <O::Inflow as Flow>::Domain: Debug,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.op.push(item)
    }
}
impl<O: RefPushOp> RefPushOp for DebugOp<O>
where
    <O::Inflow as Flow>::Domain: Debug,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.op.push(item)
    }
}
