use std::future::{self, Future, Ready};
use std::task::{Context, Poll};

use futures::future::{Either};

use super::*;

use crate::flow::*;
use crate::func::*;

/// Map-Flatten op for owned->owned values.
pub struct MapFlattenMoveOp<O: Op, F: PureFn>
where
    F::Outdomain: IntoIterator,
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

impl<O, F: PureFn> PullOp for MapFlattenMoveOp<O, F>
where
    for<'a> O: PullOp<Outflow = Df, Outdomain<'a> = F::Indomain>,
    F::Outdomain: IntoIterator,
{
    type Outflow = Df;
    type Outdomain<'s> = <F::Outdomain as IntoIterator>::Item;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
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
                            self.poll_next(ctx)
                        }
                    }
                }
            }
        }
    }
}

impl<O, F: PureFn> PushOp for MapFlattenMoveOp<O, F>
where
    F::Outdomain: IntoIterator,
    for<'a> O: PushOp<Inflow = Df, Indomain<'a> = <<F as PureFn>::Outdomain as IntoIterator>::Item>,
{
    type Inflow = Df;
    type Indomain<'p> = F::Indomain;

    type Feedback<'s> = impl Future;

    fn push<'s, 'p>(&'s mut self, item: Self::Indomain<'p>) -> Self::Feedback<'s> {
        let items = self.func.call(item);
        async move {
            let this = self;
            let mut out = Vec::new();
            for item in items {
                out.push(this.op.push(item).await);
            }
            out
        }
    }
}

/// Map-Filter op for owned->owned values.
pub struct MapFilterMoveOp<O: Op, F: PureFn> {
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

impl<T, O, F> PullOp for MapFilterMoveOp<O, F>
where
    for<'a> O: PullOp<Outflow = Df, Outdomain<'a> = F::Indomain>,
    F: PureFn<Outdomain = Option<T>>,
{
    type Outflow = Df;
    type Outdomain<'s> = T;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
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

impl<T, O, F> PushOp for MapFilterMoveOp<O, F>
where
    for<'a> O: PushOp<Inflow = Df, Indomain<'a> = T>,
    F: PureFn<Outdomain = Option<T>>,
{
    type Inflow = Df;
    type Indomain<'p> = F::Indomain;

    type Feedback<'s> = Either<O::Feedback<'s>, Ready<()>>;

    fn push<'s, 'p>(&'s mut self, item: Self::Indomain<'p>) -> Self::Feedback<'s> {
        if let Some(item) = self.func.call(item) {
            Either::Left(self.op.push(item))
        }
        else {
            Either::Right(future::ready(()))
        }
    }
}

/// Map-Fold op for ref->owned values.
pub struct MapFlattenRefOp<O: Op, F: PureRefFn> {
    op: O,
    func: F,
}
impl<O: Op, F: PureRefFn> MapFlattenRefOp<O, F> {
    pub fn new(op: O, func: F) -> Self {
        Self { op, func }
    }
}
impl<O: Op, F: PureRefFn> Op for MapFlattenRefOp<O, F> {}

impl<O, F: PureRefFn> PushOp for MapFlattenRefOp<O, F>
where
    F::Indomain: 'static,
    F::Outdomain: IntoIterator,
    for<'a> O: PushOp<Indomain<'a> = <<F as PureRefFn>::Outdomain as IntoIterator>::Item>,
{
    type Inflow = O::Inflow;
    type Indomain<'p> = &'p F::Indomain;

    type Feedback<'s> = impl Future;

    fn push<'s, 'p>(&'s mut self, item: Self::Indomain<'p>) -> Self::Feedback<'s> {
        let items = self.func.call(item);
        async move {
            let out = Vec::new();
            for item in items {
                out.push(self.op.push(item).await);
            }
            out
        }
    }
}

/// Map op for ref->ref values.
pub struct MapRefRefOp<O, F: PureRefRefFn>
where
    F::Indomain: 'static,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a F::Outdomain>,
{
    op: O,
    func: F,
}
impl<O, F: PureRefRefFn> MapRefRefOp<O, F>
where
    F::Indomain: 'static,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a F::Outdomain>,
{
    pub fn new(op: O, func: F) -> Self {
        Self { op, func }
    }
}
impl<O, F: PureRefRefFn> Op for MapRefRefOp<O, F>
where
    F::Indomain: 'static,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a F::Outdomain>,
{}

impl<O, F: PureRefRefFn> PushOp for MapRefRefOp<O, F>
where
    F::Indomain: 'static,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a F::Outdomain>,
{
    type Inflow = Rx;
    type Indomain<'p> = &'p F::Indomain;

    type Feedback<'s> = O::Feedback<'s>;

    fn push<'s, 'p>(&'s mut self, item: Self::Indomain<'p>) -> Self::Feedback<'s> {
        self.op.push(self.func.call(item))
    }
}
