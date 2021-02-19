use std::task::{ Context, Poll };

use crate::merge::Merge;

use super::op::*;
use super::flows::*;

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
impl<O: PullOp<Outflow = DF>, F: Merge<Domain = O::Outdomain>> PullOp for LatticeOp<O, F> {
    type Outflow = RX;
    type Outdomain = F::Domain;
}
impl<O: PushOp<Inflow = RX>, F: Merge<Domain = O::Indomain>> PushOp for LatticeOp<O, F> {
    type Inflow = DF;
    type Indomain = F::Domain;
}

impl<O: MovePullOp<Outflow = DF>, F: Merge<Domain = O::Outdomain>> RefPullOp for LatticeOp<O, F> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Outdomain>> {
        if let Poll::Ready(Some(item)) = self.op.poll_next(ctx) {
            F::merge_in(&mut self.state, item);
        }

        // Note: even if upstream is closed, this remains open.
        Poll::Ready(Some(&self.state))
    }
}
impl<O: RefPushOp<Inflow = RX>, F: Merge<Domain = O::Indomain>> MovePushOp for LatticeOp<O, F> {
    type Feedback = O::Feedback;

    fn push(&mut self, item: Self::Indomain) -> Self::Feedback {
        F::merge_in(&mut self.state, item);
        self.op.push(&self.state)
    }
}
