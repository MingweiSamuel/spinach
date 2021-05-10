use std::cell::RefCell;
use std::task::{Context, Poll};

use crate::lattice::{LatticeRepr, Merge, Convert};
use crate::hide::{Hide, Delta, Value};

use super::*;

/// A state-accumulating lattice op.
///
/// Input is owned `F::Domain` values as [`Df`] dataflow,
/// output is reference `&F::Domain` values as [`Rx`] reactive.
pub struct LatticeOp<O: Op, Lr: LatticeRepr + Merge<O::LatRepr>>
where
    O::LatRepr: Convert<Lr>,
{
    op: O,
    state: RefCell<Lr::Repr>,
}

impl<O: Op, Lr: LatticeRepr + Merge<O::LatRepr>> LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    pub fn new(op: O, bottom: Lr::Repr) -> Self {
        Self {
            op,
            state: RefCell::new(bottom),
        }
    }
}

impl<O: Op, Lr: LatticeRepr + Merge<O::LatRepr>> Op for LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    type LatRepr = Lr;
}

impl<O: OpDelta, Lr: LatticeRepr + Merge<O::LatRepr>> OpDelta for LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {
                let state = &mut self.state.borrow_mut();
                // F::delta(state, &mut delta); // TODO!! Doesn't minimize deltas.
                Lr::merge(state, delta.as_reveal().clone());
                Poll::Ready(Some(Hide::new(<O::LatRepr as Convert<Lr>>::convert(delta.into_reveal()))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<O: Op, Lr: LatticeRepr + Merge<O::LatRepr>> OpValue for LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        Hide::new(self.state.borrow().clone())
    }
}
