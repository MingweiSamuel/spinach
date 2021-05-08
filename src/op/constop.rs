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

impl<'s, Lr: LatticeRepr> Op<'s> for ConstOp<Lr>
where
    Lr::Repr: Clone,
{
    type LatRepr = Lr;
}

impl<'s, Lr: LatticeRepr> OpDelta<'s> for ConstOp<Lr>
where
    Lr::Repr: Clone,
{
    fn poll_delta(&'s self, _ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        Poll::Pending
    }
}

impl<'s, Lr: LatticeRepr> OpValue<'s> for ConstOp<Lr>
where
    Lr::Repr: Clone,
{
    fn get_value(&'s self) -> Hide<Value, Self::LatRepr> {
        Hide::new(self.value.clone())
    }
}
