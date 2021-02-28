use futures::future::{join_all, JoinAll};

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

    type Feedback = JoinAll<O::Feedback>;

    fn push<'p>(&mut self, item: Self::Indomain<'p>) -> Self::Feedback {
        join_all(self.func.call(item.reveal()).into_iter().map(|item| {
            let hide = Hide::ref_cast(item);
            self.op.push(hide)
        }))
    }
}
