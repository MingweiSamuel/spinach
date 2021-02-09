use std::future;
use std::task::{ Context, Poll };
use super::op::*;



pub struct NullOp<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> NullOp<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T> Op for NullOp<T> {
    type Domain = T;
    type Codomain = T;
}
impl<T> MovePullOp for NullOp<T> {
    fn poll_next(&mut self, _ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>> {
        Poll::Pending
    }
}
impl<T> RefPullOp for NullOp<T> {
    fn poll_next(&mut self, _ctx: &mut Context<'_>) -> Poll<Option<&Self::Codomain>> {
        Poll::Pending
    }
}
impl<T> MovePushOp for NullOp<T> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: Self::Domain) -> Self::Feedback {
        future::ready(())
    }
}
impl<T> RefPushOp for NullOp<T> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: &Self::Domain) -> Self::Feedback {
        future::ready(())
    }
}
