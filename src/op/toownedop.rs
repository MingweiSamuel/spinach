use std::borrow::ToOwned;
use std::task::{Context, Poll};

use super::*;

/// An Op for converting a ref flow into an owned flow via [`ToOwned`].
///
/// Supports both [`Df`] and [`Rx`].
///
/// To go from owned to ref (the opposite of this), use [`AsRefOp`].
pub struct ToOwnedOp<O: Op, T: ?Sized + ToOwned> {
    op: O,
    _phantom: std::marker::PhantomData<T>,
}
impl<O: Op, T: ?Sized + ToOwned> ToOwnedOp<O, T> {
    /// Create a CloneOp from an existing Op. Note the values received
    /// by this op must implement [`Clone`] for this op to be usable.
    pub fn new(op: O) -> Self {
        Self { op, _phantom: std::marker::PhantomData }
    }
}
impl<O: Op, T: ?Sized + ToOwned> Op for ToOwnedOp<O, T> {}
impl<O, T: ?Sized + ToOwned> PullOp for ToOwnedOp<O, T>
where
    for<'a> O: PullOp<Outdomain<'a> = &'a T>,
{
    type Outflow = O::Outflow;
    type Outdomain<'s> = T::Owned;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
        let polled = self.op.poll_next(ctx);
        polled.map(|opt| opt.map(|item| item.to_owned()))
    }
}

impl<O, T: ?Sized + ToOwned> PushOp for ToOwnedOp<O, T>
where
    for<'a> O: PushOp<Indomain<'a> = T::Owned>,
    T: 'static + Clone,
{
    type Inflow = O::Inflow;
    type Indomain<'p> = &'p T;

    type Feedback<'s> = O::Feedback<'s>;

    fn push<'s, 'p>(&'s mut self, item: Self::Indomain<'p>) -> Self::Feedback<'s> {
        self.op.push(item.to_owned())
    }
}
