use std::future::{ self, Future };
use std::task::{ Context, Poll };

use futures::future::{ Either, FutureExt };

use super::op::*;



pub struct MapFilterMoveOp<O: Op, F, U, V>
where
    F: Fn(U) -> Option<V>,
{
    op: O,
    func: F,
    _phantom: std::marker::PhantomData<( U, V )>,
}
impl<O: Op, F, U, V> MapFilterMoveOp<O, F, U, V>
where
    F: Fn(U) -> Option<V>,
{
    pub fn new(op: O, func: F) -> Self {
        Self {
            op: op,
            func: func,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<O: Op, F, U, V> Op for MapFilterMoveOp<O, F, U, V>
where
    F: Fn(U) -> Option<V>,
{
    type Domain = U;
    type Codomain = V;
}
impl<O: MovePullOp, F, V> MovePullOp for MapFilterMoveOp<O, F, O::Codomain, V>
where
    F: Fn(O::Codomain) -> Option<V>,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>> {
        let val = self.op.poll_next(ctx);
        val.map(|opt| opt.and_then(|x| (self.func)(x)))
    }
}
impl<O: MovePushOp, F, U> MovePushOp for MapFilterMoveOp<O, F, U, O::Domain>
where
    F: Fn(U) -> Option<O::Domain>,
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
