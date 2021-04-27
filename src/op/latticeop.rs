use std::cell::RefCell;
use std::task::{Context, Poll};

use crate::lattice::{Lattice};

use super::*;

pub enum LatticeWrapper<'s, F: Lattice> {
    Delta(F::Domain),
    Value(&'s RefCell<F::Domain>),
}

impl<'s, F: Lattice> Clone for LatticeWrapper<'s, F> {
    fn clone(&self) -> Self {
        match self {
            Self::Delta(delta) => Self::Delta(delta.clone()),
            Self::Value(rcell) => Self::Value(rcell),
        }
    }
}

/// A state-accumulating lattice op.
///
/// Input is owned `F::Domain` values as [`Df`] dataflow,
/// output is reference `&F::Domain` values as [`Rx`] reactive.
pub struct LatticeOp<'s, O: 's, F: 's + Lattice>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    op: O,
    state: RefCell<F::Domain>,
    _phantom: &'s (),
}

impl<'s, O, F: Lattice> LatticeOp<'s, O, F>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    pub fn new(op: O, bottom: F::Domain) -> Self {
        Self {
            op,
            state: RefCell::new(bottom),
            _phantom: &(),
        }
    }
}

impl<'s, O, F: Lattice> Op<'s> for LatticeOp<'s, O, F>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    type Outdomain = LatticeWrapper<'s, F>;
}

impl<'s, O, F: Lattice> OpDelta<'s> for LatticeOp<'s, O, F>
where
    O: OpDelta<'s, Outdomain = F::Domain>,
{
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(mut delta)) => {
                let state = &mut self.state.borrow_mut();
                F::delta(state, &mut delta); // These can be combined into one? :)
                F::merge_in(state, delta.clone());
                Poll::Ready(Some(LatticeWrapper::Delta(delta)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'s, O, F: Lattice> OpValue<'s> for LatticeOp<'s, O, F>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    fn get_value(&'s self) -> Self::Outdomain {
        LatticeWrapper::Value(&self.state)
    }
}
