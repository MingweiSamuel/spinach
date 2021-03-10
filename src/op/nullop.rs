use std::future;
use std::task::{Context, Poll};

use crate::flow::*;

use super::*;

/// An Op which does nothing. Supports both [`Df`] and [`Rx`].
///
/// If used as a push-op, pushed values are immediately dropped.
/// If used as a pull-op, never produces any values.
pub struct NullOp<F: Flow, T> {
    _phantom: std::marker::PhantomData<(F, T)>,
}
impl<F: Flow, T> NullOp<F, T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<F: Flow, T> Default for NullOp<F, T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<F: Flow, T> Op for NullOp<F, T> {}

impl<F: Flow, T> PullOp for NullOp<F, T> {
    type Outflow = F;
    type Outdomain<'s> = T;

    fn poll_next<'s>(&'s mut self, _ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
        Poll::Pending
    }
}

impl<F: Flow, T> PushOp for NullOp<F, T> {
    type Inflow = F;
    type Indomain<'p> = T;

    type Feedback<'s> = future::Ready<()>;

    fn push<'s, 'p>(&'s mut self, _item: Self::Indomain<'p>) -> Self::Feedback<'s> {
        future::ready(())
    }
}
