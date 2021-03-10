use std::future::{self, Ready};

use futures::future::{Either};

use ref_cast::RefCast;

use super::*;

use crate::lattice::Hide;
use crate::monotonic::MonotonicFilterRefFn;

/// A specific type of monotonic mapping Op.
pub struct MonotonicFilterRefOp<O, F: MonotonicFilterRefFn>
where
    F::Inmerge: 'static,
    for<'a> O: PushOp<Indomain<'a> = &'a Hide<F::Outmerge>>,
{
    op: O,
    func: F,
}

impl<O, F: MonotonicFilterRefFn> MonotonicFilterRefOp<O, F>
where
    F::Inmerge: 'static,
    for<'a> O: PushOp<Indomain<'a> = &'a Hide<F::Outmerge>>,
{
    pub fn new(op: O, func: F) -> Self {
        Self { op, func }
    }
}

impl<O, F: MonotonicFilterRefFn> Op for MonotonicFilterRefOp<O, F>
where
    F::Inmerge: 'static,
    for<'a> O: PushOp<Indomain<'a> = &'a Hide<F::Outmerge>>,
{}

impl<O, F: MonotonicFilterRefFn> PushOp for MonotonicFilterRefOp<O, F>
where
    F::Inmerge: 'static,
    for<'a> O: PushOp<Indomain<'a> = &'a Hide<F::Outmerge>>,
{
    type Inflow = O::Inflow;
    type Indomain<'p> = &'p Hide<F::Inmerge>;

    type Feedback<'s> = Either<O::Feedback<'s>, Ready<()>>;

    fn push<'s, 'p>(&'s mut self, item: Self::Indomain<'p>) -> Self::Feedback<'s> {
        if let Some(item) = self.func.call(item.reveal()) {
            let hide = Hide::ref_cast(item);
            Either::Left(self.op.push(hide))
        }
        else {
            Either::Right(future::ready(()))
        }
    }
}
