use std::task::{Context, Poll};

use ref_cast::RefCast;

use crate::flow::*;
use crate::lattice::{Hide, Lattice};

use super::*;

/// A state-accumulating lattice op.
///
/// Input is owned `F::Domain` values as [`Df`] dataflow,
/// output is reference `&F::Domain` values as [`Rx`] reactive.
pub struct LatticeOp<O: Op, F: Lattice> {
    op: O,
    state: F::Domain,
}
impl<O: Op, F: Lattice> LatticeOp<O, F> {
    /// Create a LatticeOp with the given BOTTOM value.
    pub fn new(op: O, bottom: F::Domain) -> Self {
        Self { op, state: bottom }
    }
}
impl<O: Op, F: Lattice> LatticeOp<O, F>
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

impl<O: Op, F: Lattice> Op for LatticeOp<O, F> {}
impl<O: PullOp<Outflow = Df<F::Domain>>, F: Lattice> PullOp for LatticeOp<O, F> {
    type Outflow = Rx<Hide<F>>;
}
impl<O: PushOp<Inflow = Rx<Hide<F>>>, F: Lattice> PushOp for LatticeOp<O, F> {
    type Inflow = Df<F::Domain>;
}

impl<O: MovePullOp<Outflow = Df<F::Domain>>, F: Lattice> RefPullOp for LatticeOp<O, F> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
        if let Poll::Ready(Some(item)) = self.op.poll_next(ctx) {
            F::merge_in(&mut self.state, item);
        }

        // Note: even if upstream is closed, this remains open.
        let hide = Hide::ref_cast(&self.state);
        Poll::Ready(Some(hide))
    }
}
impl<O: RefPushOp<Inflow = Rx<Hide<F>>>, F: Lattice> MovePushOp for LatticeOp<O, F> {
    type Feedback = O::Feedback;

    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        F::merge_in(&mut self.state, item);
        let hide = Hide::ref_cast(&self.state);
        self.op.push(hide)
    }
}
