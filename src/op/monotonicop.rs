use futures::future::{join_all, JoinAll};

use ref_cast::RefCast;

use super::*;

use crate::flow::*;
use crate::lattice::Hide;
use crate::monotonic::MonotonicFilterRefFn;

/// A specific type of monotonic mapping Op for [`Rx`] pipelines.
pub struct MonotonicFilterRefOp<O: Op, F: MonotonicFilterRefFn> {
    op: O,
    func: F,
}
impl<O: Op, F: MonotonicFilterRefFn> MonotonicFilterRefOp<O, F> {
    pub fn new(op: O, func: F) -> Self {
        Self { op, func }
    }
}
impl<O: Op, F: MonotonicFilterRefFn> Op for MonotonicFilterRefOp<O, F> {}

impl<O, F: MonotonicFilterRefFn> PushOp for MonotonicFilterRefOp<O, F>
where
    O: PushOp<Inflow = Rx<Hide<F::Outmerge>>>,
{
    type Inflow = Rx<Hide<F::Inmerge>>;
}
impl<O, F: MonotonicFilterRefFn> RefPushOp for MonotonicFilterRefOp<O, F>
where
    O: RefPushOp<Inflow = Rx<Hide<F::Outmerge>>>,
{
    type Feedback = JoinAll<O::Feedback>;

    fn push(&mut self, item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback {
        join_all(self.func.call(item.reveal()).into_iter().map(|item| {
            let hide = Hide::ref_cast(item);
            self.op.push(hide)
        }))
    }
}
