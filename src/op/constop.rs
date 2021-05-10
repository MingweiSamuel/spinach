use std::task::{Context, Poll};

use crate::hide::{Hide, Delta, Value};
use crate::lattice::LatticeRepr;

use super::*;

pub struct ConstOp<Lr: LatticeRepr> {
    value: Lr::Repr,
}

impl<Lr: LatticeRepr> ConstOp<Lr>
where
    Lr::Repr: Clone,
{
    pub fn new(value: Lr::Repr) -> Self {
        Self { value }
    }
}

impl<Lr: LatticeRepr> Op for ConstOp<Lr>
where
    Lr::Repr: Clone,
{
    type LatRepr = Lr;
}

impl<Lr: LatticeRepr> OpDelta for ConstOp<Lr>
where
    Lr::Repr: Clone,
{
    fn poll_delta(&self, _ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        Poll::Pending
    }
}

impl<Lr: LatticeRepr> OpValue for ConstOp<Lr>
where
    Lr::Repr: Clone,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        Hide::new(self.value.clone())
    }
}
