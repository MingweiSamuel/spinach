use std::task::{Context, Poll};

use crate::hide::{Hide, Delta};
use crate::lattice::LatticeRepr;

use super::*;

pub struct NullOp<Lr: LatticeRepr> {
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr> NullOp<Lr> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'s, Lr: LatticeRepr> Op<'s> for NullOp<Lr> {
    type LatRepr = Lr;
}

impl<'s, Lr: LatticeRepr> OpDelta<'s> for NullOp<Lr> {
    fn poll_delta(&'s self, _ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        Poll::Pending
    }
}