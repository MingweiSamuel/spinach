use std::cell::RefCell;
use std::task::{Context, Poll};

use crate::lattice::{Lattice};

use super::*;

pub enum LatticeWrapper<'s, O, F: Lattice>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    Delta {
        target: Option<&'s LatticeOp<'s, O, F>>,
        delta: Option<F::Domain>,
    },
    Value(&'s LatticeOp<'s, O, F>),
}

impl<'s, O, F: Lattice> Clone for LatticeWrapper<'s, O, F>
where
    O: Op<'s, Outdomain = F::Domain>,
    F::Domain: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Delta { target: _, delta } => Self::Delta {
                target: None, // Only first drop needs to merge.
                delta: delta.clone(),
            },
            Self::Value(target) => Self::Value(target),
        }
    }
}

impl<'s, O, F: Lattice> Drop for LatticeWrapper<'s, O, F>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    fn drop(&mut self) {
        if let Self::Delta { target, delta } = self {
            if let (Some(target), Some(delta)) = (target.take(), delta.take()) {
                F::merge_in(&mut target.state.borrow_mut(), delta);
            }
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
    type Outdomain = LatticeWrapper<'s, O, F>;
}

impl<'s, O, F: Lattice> OpDelta<'s> for LatticeOp<'s, O, F>
where
    O: OpDelta<'s, Outdomain = F::Domain>,
{
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => Poll::Ready(Some(LatticeWrapper::Delta {
                target: Some(self),
                delta: Some(delta),
            })),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'s, O, F: Lattice> OpValue<'s> for LatticeOp<'s, O, F>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    fn poll_value(&'s self, _ctx: &mut Context<'_>) -> Poll<Self::Outdomain> {
        Poll::Ready(LatticeWrapper::Value(&self))
    }
}
