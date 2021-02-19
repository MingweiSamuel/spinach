use futures::future::{ join_all, JoinAll };

use super::op::*;
use super::flow::{ Flow, RX };

use crate::monotonic::MonotonicFilterRefFn;




pub struct MonotonicFilterRefOp<O: Op, F: MonotonicFilterRefFn> {
    op: O,
    func: F,
}
impl<O: Op, F: MonotonicFilterRefFn> MonotonicFilterRefOp<O, F> {
    pub fn new(op: O, func: F) -> Self {
        Self {
            op: op,
            func: func,
        }
    }
}
impl<O: Op, F: MonotonicFilterRefFn> Op for MonotonicFilterRefOp<O, F> {}


impl<O, F: MonotonicFilterRefFn> PushOp for MonotonicFilterRefOp<O, F>
where
    O: PushOp<Inflow = RX<F::Outmerge>>,
{
    type Inflow = RX<F::Inmerge>;
}
impl<O, F: MonotonicFilterRefFn> RefPushOp for MonotonicFilterRefOp<O, F>
where
    O: RefPushOp<Inflow = RX<F::Outmerge>>,
{
    type Feedback = JoinAll<O::Feedback>;

    fn push(&mut self, item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback {
        join_all(self.func.call(item).into_iter().map(|item| self.op.push(item)))
    }
}
