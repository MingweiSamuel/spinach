use std::task::{Context, Poll};

use futures::future::{join_all, JoinAll};

use super::*;

use crate::flow::*;
use crate::func::{PureFn, PureRefFn};

/// Map-Flatten op for owned->owned values.
pub struct MapFlattenMoveOp<O: Op, F: PureFn>
where
    F::Outdomain: IntoIterator,
    <F::Outdomain as IntoIterator>::Item: 'static,
    F::Indomain: 'static,
{
    op: O,
    func: F,
    /// A value produced by func, to be iterated in pull-based pipes.
    out: Option<<<F as PureFn>::Outdomain as IntoIterator>::IntoIter>,
}
impl<O: Op, F: PureFn> MapFlattenMoveOp<O, F>
where
    F::Outdomain: IntoIterator,
{
    pub fn new(op: O, func: F) -> Self {
        Self {
            op,
            func,
            out: None,
        }
    }
}
impl<O: Op, F: PureFn> Op for MapFlattenMoveOp<O, F> where F::Outdomain: IntoIterator {}

impl<O: PullOp<Outflow = Df<F::Indomain>>, F: PureFn> PullOp for MapFlattenMoveOp<O, F>
where
    F::Outdomain: IntoIterator,
{
    type Outflow = Df<<F::Outdomain as IntoIterator>::Item>;
}
impl<O: MovePullOp<Outflow = Df<F::Indomain>>, F: PureFn> MovePullOp for MapFlattenMoveOp<O, F>
where
    F::Outdomain: IntoIterator,
{
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        match self.out.as_mut().and_then(|out| out.next()) {
            Some(item) => Poll::Ready(Some(item)),
            None => {
                match self.op.poll_next(ctx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(None) => Poll::Ready(None), // EOS
                    Poll::Ready(Some(item)) => {
                        let mut newout = self.func.call(item).into_iter();
                        if let Some(outitem) = newout.next() {
                            Poll::Ready(Some(outitem))
                        } else {
                            Poll::Pending
                        }
                    }
                }
            }
        }
        // let val = self.op.poll_next(ctx);
        // val.map(|opt| opt.and_then(|x| self.func.call(x)))
    }
}

impl<O, F: PureFn> PushOp for MapFlattenMoveOp<O, F>
where
    F::Outdomain: IntoIterator,
    O: PushOp<Inflow = Df<<<F as PureFn>::Outdomain as IntoIterator>::Item>>,
{
    type Inflow = Df<F::Indomain>;
}
impl<O, F: PureFn> MovePushOp for MapFlattenMoveOp<O, F>
where
    F::Outdomain: IntoIterator,
    O: MovePushOp<Inflow = Df<<<F as PureFn>::Outdomain as IntoIterator>::Item>>,
{
    type Feedback = JoinAll<O::Feedback>;

    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        join_all(
            self.func
                .call(item)
                .into_iter()
                .map(|item| self.op.push(item)),
        )
    }
}

/// Map-Filter op for owned->owned values.
pub struct MapFilterMoveOp<O: Op, F: PureFn>
where
    F::Indomain: 'static,
{
    op: O,
    func: F,
}
impl<O: Op, F: PureFn> MapFilterMoveOp<O, F> {
    pub fn new(op: O, func: F) -> Self {
        Self { op, func }
    }
}
impl<O: Op, F: PureFn> Op for MapFilterMoveOp<O, F> {}

// impl<O: PullOp<Outflow = Df<F::Indomain>>, F: PureFn> PullOp for MapFlattenMoveOp<O, F>
// where
//     <F as PureFn>::Outdomain: IntoIterator,
// {
//     type Outflow = Df<<F::Outdomain as IntoIterator>::Item>;
// }

impl<T: 'static, O, F> PullOp for MapFilterMoveOp<O, F>
where
    O: PullOp<Outflow = Df<F::Indomain>>,
    F: PureFn<Outdomain = Option<T>>,
{
    type Outflow = Df<T>;
}
impl<T: 'static, O, F> MovePullOp for MapFilterMoveOp<O, F>
where
    O: MovePullOp<Outflow = Df<F::Indomain>>,
    F: PureFn<Outdomain = Option<T>>,
{
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        match self.op.poll_next(ctx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None), // EOS
            Poll::Ready(Some(item)) => match self.func.call(item) {
                Some(item) => Poll::Ready(Some(item)),
                None => Poll::Pending,
            },
        }
    }
}

impl<T: 'static, O, F> PushOp for MapFilterMoveOp<O, F>
where
    O: PushOp<Inflow = Df<T>>,
    F: PureFn<Outdomain = Option<T>>,
{
    type Inflow = Df<F::Indomain>;
}
impl<T: 'static, O, F> MovePushOp for MapFilterMoveOp<O, F>
where
    O: MovePushOp<Inflow = Df<T>>,
    F: PureFn<Outdomain = Option<T>>,
{
    type Feedback = JoinAll<O::Feedback>;

    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        join_all(
            self.func
                .call(item)
                .into_iter()
                .map(|item| self.op.push(item)),
        )
    }
}

/// Map-Fold op for ref->owned values.
pub struct MapFoldRefOp<O: Op, F: PureRefFn>
where
    F::Indomain: 'static,
{
    op: O,
    func: F,
}
impl<O: Op, F: PureRefFn> MapFoldRefOp<O, F> {
    pub fn new(op: O, func: F) -> Self {
        Self { op, func }
    }
}
impl<O: Op, F: PureRefFn> Op for MapFoldRefOp<O, F> {}

impl<O, F: PureRefFn> PushOp for MapFoldRefOp<O, F>
where
    F::Outdomain: IntoIterator,
    O: PushOp<Inflow = Df<<<F as PureRefFn>::Outdomain as IntoIterator>::Item>>,
    <F::Outdomain as std::iter::IntoIterator>::Item: 'static,
{
    type Inflow = Df<F::Indomain>;
}
impl<O, F: PureRefFn> RefPushOp for MapFoldRefOp<O, F>
where
    F::Outdomain: IntoIterator,
    O: MovePushOp<Inflow = Df<<<F as PureRefFn>::Outdomain as IntoIterator>::Item>>,
    <F::Outdomain as std::iter::IntoIterator>::Item: 'static,
{
    type Feedback = JoinAll<O::Feedback>;

    fn push(&mut self, item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback {
        join_all(
            self.func
                .call(item)
                .into_iter()
                .map(|item| self.op.push(item)),
        )
    }
}
