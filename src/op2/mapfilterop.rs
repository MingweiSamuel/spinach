use std::future::{ self, Future };
use std::task::{ Context, Poll };

use futures::future::{ Either, FutureExt };

use super::op::*;
use super::util::{ PureFn, PureRefFn };



pub struct MapFilterMoveOp<O: Op, F: PureFn, T> {
    op: O,
    func: F,
    _phantom: std::marker::PhantomData<T>,
}
impl<O: Op, F: PureFn, T> MapFilterMoveOp<O, F, T> {
    pub fn new(op: O, func: F) -> Self {
        Self {
            op: op,
            func: func,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<O: Op, F: PureFn, T> Op for MapFilterMoveOp<O, F, T> {}
impl<O: PullOp, F, T> PullOp for MapFilterMoveOp<O, F, T>
where
    F: PureFn<Domain = O::Codomain, Codomain = Option<T>>,
{
    type Outflow = O::Outflow;
    type Codomain = T;
}
impl<O: PushOp, F, T> PushOp for MapFilterMoveOp<O, F, T>
where
    F: PureFn<Domain = T, Codomain = Option<O::Domain>>,
{
    type Inflow = O::Inflow;
    type Domain = T;
}
impl<O: MovePullOp, F, T> MovePullOp for MapFilterMoveOp<O, F, T>
where
    F: PureFn<Domain = O::Codomain, Codomain = Option<T>>,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>> {
        let val = self.op.poll_next(ctx);
        val.map(|opt| opt.and_then(|x| self.func.call(x)))
    }
}
impl<O: MovePushOp, F, T> MovePushOp for MapFilterMoveOp<O, F, T>
where
    F: PureFn<Domain = T, Codomain = Option<O::Domain>>,
{
    type Feedback = impl Future;

    fn push(&mut self, item: Self::Domain) -> Self::Feedback {
        if let Some(item) = self.func.call(item) {
            Either::Left(self.op.push(item).map(|x| Some(x)))
        }
        else {
            Either::Right(future::ready(None))
        }
    }
}




pub struct MapFilterRefOp<O: Op, F: PureRefFn, T> {
    op: O,
    func: F,
    _phantom: std::marker::PhantomData<T>,
}
impl<O: Op, F: PureRefFn, T> MapFilterRefOp<O, F, T> {
    pub fn new(op: O, func: F) -> Self {
        Self {
            op: op,
            func: func,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<O: Op, F: PureRefFn, T> Op for MapFilterRefOp<O, F, T> {}
impl<O: PullOp, F, T> PullOp for MapFilterRefOp<O, F, T>
where
    F: PureRefFn<Domain = O::Codomain, Codomain = Option<T>>,
{
    type Outflow = O::Outflow;
    type Codomain = T;
}
impl<O: PushOp, F, T> PushOp for MapFilterRefOp<O, F, T>
where
    F: PureRefFn<Domain = T, Codomain = Option<O::Domain>>,
{
    type Inflow = O::Inflow;
    type Domain = T;
}
// impl<O: RefPullOp, F, T> MovePullOp for MapFilterRefOp<O, F, T>
// where
//     F: Fn(&O::Codomain) -> Option<T>,
// {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>> {
//         let val = self.op.poll_next(ctx);
//         val.map(|opt| opt.and_then(|x| (self.func)(x)))
//     }
// }
impl<O: MovePushOp, F, T> RefPushOp for MapFilterRefOp<O, F, T>
where
    F: PureRefFn<Domain = T, Codomain = Option<O::Domain>>,
{
    type Feedback = impl Future;

    fn push(&mut self, item: &Self::Domain) -> Self::Feedback {
        if let Some(item) = self.func.call(item) {
            Either::Left(self.op.push(item).map(|x| Some(x)))
        }
        else {
            Either::Right(future::ready(None))
        }
    }
}
