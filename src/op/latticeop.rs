use std::task::{Context, Poll};

use crate::merge::Merge;

use super::*;

/// A state-accumulating lattice op.
///
/// Input is owned `F::Domain` values as [`Df`] dataflow,
/// output is reference `&F::Domain` values as [`Rx`] reactive.
pub struct LatticeOp<O: Op, F: Merge> {
    op: O,
    state: F::Domain,
}
impl<O: Op, F: Merge> LatticeOp<O, F> {
    /// Create a LatticeOp with the given BOTTOM value.
    pub fn new(op: O, bottom: F::Domain) -> Self {
        Self { op, state: bottom }
    }
}
impl<O: Op, F: Merge> LatticeOp<O, F>
where
    F::Domain: Default,
{
    /// Create a LatticeOp using the default value as bottom.
    pub fn new_default(op: O) -> Self {
        Self {
            op,
            state: Default::default(),
        }
    }
}

impl<O: Op, F: Merge> Op for LatticeOp<O, F> {}
impl<O: PullOp<Outflow = Df<F::Domain>>, F: Merge> PullOp for LatticeOp<O, F> {
    type Outflow = Rx<F>;
}
impl<O: PushOp<Inflow = Rx<F>>, F: Merge> PushOp for LatticeOp<O, F> {
    type Inflow = Df<F::Domain>;
}

impl<O: MovePullOp<Outflow = Df<F::Domain>>, F: Merge> RefPullOp for LatticeOp<O, F> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
        if let Poll::Ready(Some(item)) = self.op.poll_next(ctx) {
            F::merge_in(&mut self.state, item);
        }

        // Note: even if upstream is closed, this remains open.
        Poll::Ready(Some(&self.state))
    }
}
impl<O: RefPushOp<Inflow = Rx<F>>, F: Merge> MovePushOp for LatticeOp<O, F> {
    type Feedback = O::Feedback;

    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        F::merge_in(&mut self.state, item);
        self.op.push(&self.state)
    }
}
