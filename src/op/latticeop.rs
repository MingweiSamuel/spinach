use std::task::{Context, Poll};

use ref_cast::RefCast;

use crate::flow::*;
use crate::lattice::{Hide, Lattice};

use super::*;

/// A state-accumulating lattice op.
///
/// Input is owned `F::Domain` values as [`Df`] dataflow,
/// output is reference `&F::Domain` values as [`Rx`] reactive.
pub struct LatticeOp<O: Op, F: 'static + Lattice> {
    op: O,
    state: F::Domain,
}
impl<O: Op, F: 'static + Lattice> LatticeOp<O, F> {
    /// Create a LatticeOp with the given BOTTOM value.
    pub fn new(op: O, bottom: F::Domain) -> Self {
        Self { op, state: bottom }
    }
}
impl<O: Op, F: 'static + Lattice> LatticeOp<O, F>
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

impl<O: Op, F: 'static + Lattice> Op for LatticeOp<O, F> {}

impl<O, F: 'static + Lattice> PullOp for LatticeOp<O, F>
where
    for<'a> O: PullOp<Outflow = Df, Outdomain<'a> = F::Domain>,
{
    type Outflow = Rx;
    type Outdomain<'p> = &'p Hide<F>;

    fn poll_next<'p>(&'p mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'p>>> {
        if let Poll::Ready(Some(item)) = self.op.poll_next(ctx) {
            F::merge_in(&mut self.state, item);
        }

        // Note: even if upstream is closed, this remains open.
        let hide = Hide::ref_cast(&self.state);
        Poll::Ready(Some(hide))
    }
}

impl<O, F: 'static + Lattice> PushOp for LatticeOp<O, F>
where
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a Hide<F>>,
{
    type Inflow = Df;
    type Indomain<'p> = F::Domain;

    type Feedback = O::Feedback;

    fn push<'p>(&mut self, item: Self::Indomain<'p>) -> Self::Feedback {
        F::merge_in(&mut self.state, item);
        let hide = Hide::ref_cast(&self.state);
        self.op.push(hide)
    }
}
