use std::fmt::Debug;
use std::task::{Context, Poll};

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

impl<O, T> PullOp for DebugOp<O>
where
    T: Debug,
    for<'a> O: PullOp<Outdomain<'a> = T>,
{
    type Outflow = O::Outflow;
    type Outdomain<'s> = O::Outdomain<'s>;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
        let polled = self.op.poll_next(ctx);
        if let Poll::Ready(Some(item)) = &polled {
            println!("{}: {:?}", self.tag, item);
        }
        polled
    }
}

impl<O, T> PushOp for DebugOp<O>
where
    T: Debug,
    for<'a> O: PushOp<Indomain<'a> = T>,
{
    type Inflow = O::Inflow;
    type Indomain<'p> = O::Indomain<'p>;

    type Feedback = O::Feedback;

    fn push<'p>(&mut self, item: Self::Indomain<'p>) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.op.push(item)
    }
}
