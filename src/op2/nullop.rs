use std::future;
use std::task::{ Context, Poll };

use super::op::*;
use super::types::*;


pub struct NullOp<F: Flow, T> {
    _phantom: std::marker::PhantomData<( F, T )>,
}
impl<F: Flow, T> NullOp<F, T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<F: Flow, T> Op for NullOp<F, T> {}
impl<F: Flow, T> PullOp for NullOp<F, T> {
    type Outflow = F;
    type Outdomain = T;
}
impl <F: Flow, T> PushOp for NullOp<F, T> {
    type Inflow = F;
    type Indomain = T;
}
impl<F: Flow, T> MovePullOp for NullOp<F, T> {
    fn poll_next(&mut self, _ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        Poll::Pending
    }
}
impl<F: Flow, T> RefPullOp for NullOp<F, T> {
    fn poll_next(&mut self, _ctx: &mut Context<'_>) -> Poll<Option<&Self::Outdomain>> {
        Poll::Pending
    }
}
impl<F: Flow, T> MovePushOp for NullOp<F, T> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: Self::Indomain) -> Self::Feedback {
        future::ready(())
    }
}
impl<F: Flow, T> RefPushOp for NullOp<F, T> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: &Self::Indomain) -> Self::Feedback {
        future::ready(())
    }
}
