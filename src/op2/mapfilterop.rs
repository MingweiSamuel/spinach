use std::task::{ Context, Poll };

use futures::future::{ join_all, JoinAll };

use super::op::*;
use super::util::{ PureFn, PureRefFn };



pub struct MapFlattenMoveOp<O: Op, F: PureFn, T>
where
    <F as PureFn>::Outdomain: IntoIterator,
{
    op: O,
    func: F,
    /// A value produced by func, to be iterated in pull-based pipes.
    out: Option<<F::Outdomain as IntoIterator>::IntoIter>,
    _phantom: std::marker::PhantomData<T>,
}
impl<O: Op, F: PureFn, T> MapFlattenMoveOp<O, F, T>
where
    <F as PureFn>::Outdomain: IntoIterator,
{
    pub fn new(op: O, func: F) -> Self {
        Self {
            op: op,
            func: func,
            out: None,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<O: Op, F: PureFn, T> Op for MapFlattenMoveOp<O, F, T>
where
    <F as PureFn>::Outdomain: IntoIterator,
{}
impl<O: PullOp, F, T> PullOp for MapFlattenMoveOp<O, F, T>
where
    F: PureFn<Indomain = O::Outdomain>,
    <F as PureFn>::Outdomain: IntoIterator<Item = T>,
{
    type Outflow = O::Outflow;
    type Outdomain = T;
}
impl<O: PushOp, F, T> PushOp for MapFlattenMoveOp<O, F, T>
where
    F: PureFn<Indomain = T>,
    <F as PureFn>::Outdomain: IntoIterator<Item = O::Indomain>,
{
    type Inflow = O::Inflow;
    type Indomain = T;
}
impl<O: MovePullOp, F, T> MovePullOp for MapFlattenMoveOp<O, F, T>
where
    F: PureFn<Indomain = O::Outdomain>,
    <F as PureFn>::Outdomain: IntoIterator<Item = T>,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        match self.out.as_mut().and_then(|out| out.next()) {
            Some(val) => Poll::Ready(Some(val)),
            None => {
                match self.op.poll_next(ctx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(None) => Poll::Ready(None), // EOS
                    Poll::Ready(Some(item)) => {
                        let mut newout = self.func.call(item).into_iter();
                        if let Some(outitem) = newout.next() {
                            Poll::Ready(Some(outitem))
                        }
                        else {
                            Poll::Pending
                        }
                    },
                }
            },
        }
        // let val = self.op.poll_next(ctx);
        // val.map(|opt| opt.and_then(|x| self.func.call(x)))
    }
}
impl<O: MovePushOp, F, T> MovePushOp for MapFlattenMoveOp<O, F, T>
where
    F: PureFn<Indomain = T>,
    <F as PureFn>::Outdomain: IntoIterator<Item = O::Indomain>,
{
    type Feedback = JoinAll<O::Feedback>;

    fn push(&mut self, item: Self::Indomain) -> Self::Feedback {
        join_all(self.func.call(item).into_iter().map(|item| self.op.push(item)))
    }
}




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
    F: PureFn<Indomain = O::Outdomain, Outdomain = Option<T>>,
{
    type Outflow = O::Outflow;
    type Outdomain = T;
}
impl<O: PushOp, F, T> PushOp for MapFilterMoveOp<O, F, T>
where
    F: PureFn<Indomain = T, Outdomain = Option<O::Indomain>>,
{
    type Inflow = O::Inflow;
    type Indomain = T;
}
impl<O: MovePullOp, F, T> MovePullOp for MapFilterMoveOp<O, F, T>
where
    F: PureFn<Indomain = O::Outdomain, Outdomain = Option<T>>,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        match self.op.poll_next(ctx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None), // EOS
            Poll::Ready(Some(item)) => {
                match self.func.call(item) {
                    Some(item) => Poll::Ready(Some(item)),
                    None => Poll::Pending,
                }
            },
        }
    }
}
impl<O: MovePushOp, F, T> MovePushOp for MapFilterMoveOp<O, F, T>
where
    F: PureFn<Indomain = T, Outdomain = Option<O::Indomain>>,
{
    type Feedback = JoinAll<O::Feedback>;

    fn push(&mut self, item: Self::Indomain) -> Self::Feedback {
        join_all(self.func.call(item).into_iter().map(|item| self.op.push(item)))
    }
}




pub struct MapFoldRefOp<O: Op, F: PureRefFn, T> {
    op: O,
    func: F,
    _phantom: std::marker::PhantomData<T>,
}
impl<O: Op, F: PureRefFn, T> MapFoldRefOp<O, F, T> {
    pub fn new(op: O, func: F) -> Self {
        Self {
            op: op,
            func: func,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<O: Op, F: PureRefFn, T> Op for MapFoldRefOp<O, F, T> {}
impl<O: PushOp, F, T> PushOp for MapFoldRefOp<O, F, T>
where
    F: PureRefFn<Indomain = T>,
    <F as PureRefFn>::Outdomain: IntoIterator<Item = O::Indomain>,
{
    type Inflow = O::Inflow;
    type Indomain = T;
}
impl<O: MovePushOp, F, T> RefPushOp for MapFoldRefOp<O, F, T>
where
    F: PureRefFn<Indomain = T>,
    <F as PureRefFn>::Outdomain: IntoIterator<Item = O::Indomain>,
{
    type Feedback = JoinAll<O::Feedback>;

    fn push(&mut self, item: &Self::Indomain) -> Self::Feedback {
        join_all(self.func.call(item).into_iter().map(|item| self.op.push(item)))
    }
}
