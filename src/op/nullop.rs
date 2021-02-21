use std::future;
use std::task::{ Context, Poll };

use super::op::*;
use super::flow::*;


pub struct NullOp<F: Flow> {
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Flow> NullOp<F> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F: Flow> Op for NullOp<F> {}

impl<'slf, F: Flow> PullOp<'slf> for NullOp<F> {
    type Outflow<'a> = F;

    fn poll_next<'a>(&'slf mut self, _ctx: &mut Context<'_>) -> Poll<Option<<Self::Outflow<'a> as Flow>::Domain>> {
        Poll::Pending
    }
}

impl<'slf, F: Flow> PushOp<'slf> for NullOp<F> {
    type Inflow<'a> = F;
    type Feedback<'a, 's> = future::Ready<()>;

    #[must_use]
    fn push<'a>(&'slf mut self, item: <Self::Inflow<'a> as Flow>::Domain) -> Self::Feedback<'a, 'slf> {
        future::ready(())
    }
}
