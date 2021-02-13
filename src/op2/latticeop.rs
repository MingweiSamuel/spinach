use std::task::{ Context, Poll };

use crate::merge::Merge;

use super::op::*;
use super::types::*;

pub struct LatticeOp<O: Op, F: Merge> {
    op: O,
    state: F::Domain,
}
impl<O: Op, F: Merge> LatticeOp<O, F> {
    pub fn new(op: O, bottom: F::Domain) -> Self {
        LatticeOp {
            op: op,
            state: bottom,
        }
    }
}
impl<O: Op, F: Merge> Op for LatticeOp<O, F> {}
impl<O: PullOp<Outflow = DF>, F: Merge<Domain = O::Codomain>> PullOp for LatticeOp<O, F> {
    type Outflow = RX;
    type Codomain = F::Domain;
}
impl<O: PushOp<Inflow = RX>, F: Merge<Domain = O::Domain>> PushOp for LatticeOp<O, F> {
    type Inflow = DF;
    type Domain = F::Domain;
}

impl<O: MovePullOp<Outflow = DF>, F: Merge<Domain = O::Codomain>> RefPullOp for LatticeOp<O, F> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Codomain>> {
        if let Poll::Ready(Some(item)) = self.op.poll_next(ctx) {
            F::merge_in(&mut self.state, item);
        }

        // Note: even if upstream is closed, this remains open.
        Poll::Ready(Some(&self.state))
    }
}
impl<O: RefPushOp<Inflow = RX>, F: Merge<Domain = O::Domain>> MovePushOp for LatticeOp<O, F> {
    type Feedback = O::Feedback;

    fn push(&mut self, item: Self::Domain) -> Self::Feedback {
        F::merge_in(&mut self.state, item);
        self.op.push(&self.state)
    }
}
