use futures::future::{join_all, JoinAll};

use super::*;

use crate::flow::*;
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
    O: PushOp<Inflow = Rx<F::Outmerge>>,
{
    type Inflow = Rx<F::Inmerge>;
}
impl<O, F: MonotonicFilterRefFn> RefPushOp for MonotonicFilterRefOp<O, F>
where
    O: RefPushOp<Inflow = Rx<F::Outmerge>>,
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
