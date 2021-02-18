use std::task::{ Context, Poll };

use crate::merge::{ Merge, Sealed };

use super::op::*;
use super::types::*;

pub struct LatticeOp<O: Op, F: Merge> {
    op: O,
    state: Sealed<F>,
}
impl<O: Op, F: Merge> LatticeOp<O, F> {
    pub fn new(op: O, bottom: F::Domain) -> Self {
        Self {
            op: op,
            state: Sealed::new(bottom),
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
            state: Sealed::new(Default::default()),
        }
    }
}

impl<O: Op, F: Merge> Op for LatticeOp<O, F> {}
impl<O: PullOp<Outflow = DF, Outdomain = F::Domain>, F: Merge> PullOp for LatticeOp<O, F> {
    type Outflow = RX;
    type Outdomain = Sealed<F>;
}
impl<O: PushOp<Inflow = RX, Indomain = Sealed<F>>, F: Merge> PushOp for LatticeOp<O, F> {
    type Inflow = DF;
    type Indomain = F::Domain;
}

impl<I: MovePullOp<Outflow = DF, Outdomain = F::Domain>, F: Merge> RefPullOp for LatticeOp<I, F> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Outdomain>> {
        if let Poll::Ready(Some(item)) = self.op.poll_next(ctx) {
            self.state.merge_in(item);
        }

        // Note: even if upstream is closed, this remains open.
        Poll::Ready(Some(&self.state))
    }
}
impl<O: RefPushOp<Inflow = RX, Indomain = Sealed<F>>, F: Merge> MovePushOp for LatticeOp<O, F> {
    type Feedback = O::Feedback;

    fn push(&mut self, item: Self::Indomain) -> Self::Feedback {
        self.state.merge_in(item);
        self.op.push(&self.state)
    }
}
