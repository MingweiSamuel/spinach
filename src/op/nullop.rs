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
impl<F: Flow> PullOp for NullOp<F> {
    type Outflow = F;
}
impl <F: Flow> PushOp for NullOp<F> {
    type Inflow = F;
}
impl<F: Flow> MovePullOp for NullOp<F> {
    fn poll_next(&mut self, _ctx: &mut Context<'_>) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        Poll::Pending
    }
}
impl<F: Flow> RefPullOp for NullOp<F> {
    fn poll_next(&mut self, _ctx: &mut Context<'_>) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
        Poll::Pending
    }
}
impl<F: Flow> MovePushOp for NullOp<F> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        future::ready(())
    }
}
impl<F: Flow> RefPushOp for NullOp<F> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback {
        future::ready(())
    }
}
