use std::future::Future;
use std::task::{Context, Poll};

use tokio::join;

use crate::flow::Df;

use super::*;

/// An Op which, for each element, copies it and pushes to **both** downstream ops.
/// Note: [`Copy`] is lightweight and implemented for references and simple primitives. It is not [`Clone`].
pub struct SplitOp<O, P>
where
    O: PushOp<Inflow = Df>,
    for<'a> O::Indomain<'a>: Copy,
    for<'a> P: PushOp<Inflow = Df, Indomain<'a> = O::Indomain<'a>>,
{
    op0: O,
    op1: P,
}

impl<O, P> SplitOp<O, P>
where
    O: PushOp<Inflow = Df>,
    for<'a> O::Indomain<'a>: Copy,
    for<'a> P: PushOp<Inflow = Df, Indomain<'a> = O::Indomain<'a>>,
{
    /// Split op to op0 and op1.
    pub fn new(op0: O, op1: P) -> Self {
        Self { op0, op1 }
    }
}

impl<O, P> Op for SplitOp<O, P>
where
    O: PushOp<Inflow = Df>,
    for<'a> O::Indomain<'a>: Copy,
    for<'a> P: PushOp<Inflow = Df, Indomain<'a> = O::Indomain<'a>>,
{}

impl<O, P> PushOp for SplitOp<O, P>
where
    O: PushOp<Inflow = Df>,
    for<'a> O::Indomain<'a>: Copy,
    for<'a> P: PushOp<Inflow = Df, Indomain<'a> = O::Indomain<'a>>,
{
    type Inflow = Df;
    type Indomain<'p> = O::Indomain<'p>;

    type Feedback = impl Future;

    fn push<'p>(&mut self, item: Self::Indomain<'p>) -> Self::Feedback {
        let f0 = self.op0.push(item);
        let f1 = self.op1.push(item);
        async move { join!(f0, f1) }
    }
}

/// An Op which receives from two upstream ops.
///
/// Note that this is biased, it will give priority to the first op, then the second op.
pub struct MergeOp<O, P>
where
    O: PullOp<Outflow = Df>,
    for<'a> P: PullOp<Outflow = Df, Outdomain<'a> = O::Outdomain<'a>>,
{
    op0: O,
    op1: P,
}

impl<O, P> MergeOp<O, P>
where
    O: PullOp<Outflow = Df>,
    for<'a> P: PullOp<Outflow = Df, Outdomain<'a> = O::Outdomain<'a>>,
{
    /// Receive from both OP0 and OP1, where priority is to pull from OP0.
    pub fn new(op0: O, op1: P) -> Self {
        Self { op0, op1 }
    }
}

impl<O, P> Op for MergeOp<O, P>
where
    O: PullOp<Outflow = Df>,
    for<'a> P: PullOp<Outflow = Df, Outdomain<'a> = O::Outdomain<'a>>,
{}

impl<O, P> PullOp for MergeOp<O, P>
where
    O: PullOp<Outflow = Df>,
    for<'a> P: PullOp<Outflow = Df, Outdomain<'a> = O::Outdomain<'a>>,
{
    type Outflow = O::Outflow;
    type Outdomain<'s> = O::Outdomain<'s>;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
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
