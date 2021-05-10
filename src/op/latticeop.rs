use std::cell::RefCell;
use std::task::{Context, Poll};

use crate::lattice::{LatticeRepr, Merge, Convert};
use crate::hide::{Hide, Delta, Value};

use super::*;

/// A state-accumulating lattice op.
///
/// Input is owned `F::Domain` values as [`Df`] dataflow,
/// output is reference `&F::Domain` values as [`Rx`] reactive.
pub struct LatticeOp<O, Lr: LatticeRepr> {
    op: O,
    state: RefCell<Lr::Repr>,
}

impl<'s, O: Op<'s>, Lr: LatticeRepr + Merge<O::LatRepr>> LatticeOp<O, Lr>
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

impl<'s, O: Op<'s>, Lr: LatticeRepr + Merge<O::LatRepr>> Op<'s> for LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    type LatRepr = Lr;
}

impl<'s, O: OpDelta<'s>, Lr: LatticeRepr + Merge<O::LatRepr>> OpDelta<'s> for LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
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

impl<'s, O: Op<'s>, Lr: LatticeRepr + Merge<O::LatRepr>> OpValue<'s> for LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    fn get_value(&'s self) -> Hide<Value, Self::LatRepr> {
        Hide::new(self.state.borrow().clone())
    }
}
