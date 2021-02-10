use std::future::{ self, Future };
use std::task::{ Context, Poll };

use futures::future::{ Either, FutureExt };

use super::op::*;



pub struct MapFilterMoveOp<O: Op, F, T> {
    op: O,
    func: F,
    _phantom: std::marker::PhantomData<T>,
}
impl<O: Op, F, T> MapFilterMoveOp<O, F, T> {
    pub fn new(op: O, func: F) -> Self {
        Self {
            op: op,
            func: func,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<O: Op, F, T> Op for MapFilterMoveOp<O, F, T> {}
impl<O: PullOp, F, T> PullOp for MapFilterMoveOp<O, F, T>
where
    F: Fn(O::Codomain) -> Option<T>,
{
    type Codomain = T;
}
impl<O: PushOp, F, T> PushOp for MapFilterMoveOp<O, F, T>
where
    F: Fn(T) -> Option<O::Domain>,
{
    type Domain = T;
}
impl<O: MovePullOp, F, T> MovePullOp for MapFilterMoveOp<O, F, T>
where
    F: Fn(O::Codomain) -> Option<T>,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>> {
        let val = self.op.poll_next(ctx);
        val.map(|opt| opt.and_then(|x| (self.func)(x)))
    }
}
impl<O: MovePushOp, F, T> MovePushOp for MapFilterMoveOp<O, F, T>
where
    F: Fn(T) -> Option<O::Domain>,
{
    type Feedback = impl Future;

    fn push(&mut self, item: Self::Domain) -> Self::Feedback {
        if let Some(item) = (self.func)(item) {
            Either::Left(self.op.push(item).map(|x| Some(x)))
        }
        else {
            Either::Right(future::ready(None))
        }
    }
}
