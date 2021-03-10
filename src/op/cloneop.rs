use std::task::{Context, Poll};

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
impl<T, O> PullOp for CloneOp<O>
where
    for<'a> O: PullOp<Outdomain<'a> = &'a T>,
    T: Clone,
{
    type Outflow = O::Outflow;
    type Outdomain<'s> = T;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
        let polled = self.op.poll_next(ctx);
        polled.map(|opt| opt.cloned())
    }
}

impl<T, O> PushOp for CloneOp<O>
where
    for<'a> O: PushOp<Indomain<'a> = T>,
    T: 'static + Clone,
{
    type Inflow = O::Inflow;
    type Indomain<'p> = &'p T;

    type Feedback<'s> = O::Feedback<'s>;

    fn push<'s, 'p>(&'s mut self, item: Self::Indomain<'p>) -> Self::Feedback<'s> {
        self.op.push(item.clone())
    }
}
