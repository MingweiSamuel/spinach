use std::task::{ Context, Poll };

use crate::merge::Merge;

use super::op::*;
use super::flow::*;

pub struct LatticeOp<O: Op, F: Merge> {
    op: O,
    state: F::Domain,
}
impl<O: Op, F: Merge> LatticeOp<O, F> {
    pub fn new(op: O, bottom: F::Domain) -> Self {
        Self {
            op: op,
            state: bottom,
        }
    }
}
impl<O: Op, F: Merge> LatticeOp<O, F>
where
    F::Domain: Default,
{
    pub fn new_default(op: O) -> Self {
        Self {
            op: op,
            state: Default::default(),
        }
    }
}

impl<O: Op, F: Merge> Op for LatticeOp<O, F> {}
impl<O: PullOp<Outflow = DF<F::Domain>>, F: Merge> PullOp for LatticeOp<O, F> {
    type Outflow = RX<F>;
}
impl<O: PushOp<Inflow = RX<F>>, F: Merge> PushOp for LatticeOp<O, F> {
    type Inflow = DF<F::Domain>;
}

impl<O: MovePullOp<Outflow = DF<F::Domain>>, F: Merge> RefPullOp for LatticeOp<O, F> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
        if let Poll::Ready(Some(item)) = self.op.poll_next(ctx) {
            F::merge_in(&mut self.state, item);
        }

        // Note: even if upstream is closed, this remains open.
        Poll::Ready(Some(&self.state))
    }
}
impl<O: RefPushOp<Inflow = RX<F>>, F: Merge> MovePushOp for LatticeOp<O, F> {
    type Feedback = O::Feedback;

    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        F::merge_in(&mut self.state, item);
        self.op.push(&self.state)
    }
}
