use std::task::{Context, Poll};

use crate::hide::{Hide, Delta, Value};
use crate::lattice::{LatticeRepr};

use super::*;

pub struct MorphOp<O: Op, Lr: LatticeRepr, F>
where
    F: Fn(Hide<Delta, O::LatRepr>) -> Hide<Delta, Lr>,
{
    op: O,
    f: F,
}

impl<O: Op, Lr: LatticeRepr, F> MorphOp<O, Lr, F>
where
    F: Fn(Hide<Delta, O::LatRepr>) -> Hide<Delta, Lr>,
{
    pub fn new(op: O, f: F) -> Self {
        Self { op, f }
    }
}

impl<O: Op, Lr: LatticeRepr, F> Op for MorphOp<O, Lr, F>
where
    F: Fn(Hide<Delta, O::LatRepr>) -> Hide<Delta, Lr>,
{
    type LatRepr = Lr;
}

impl<O: OpDelta, Lr: LatticeRepr, F> OpDelta for MorphOp<O, Lr, F>
where
    F: Fn(Hide<Delta, O::LatRepr>) -> Hide<Delta, Lr>,
{
    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => Poll::Ready(Some((self.f)(delta))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<O: OpValue, Lr: LatticeRepr, F> OpValue for MorphOp<O, Lr, F>
where
    F: Fn(Hide<Delta, O::LatRepr>) -> Hide<Delta, Lr>,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        let value: Hide<Value, O::LatRepr> = self.op.get_value();
        (self.f)(value.into_delta()).into_reveal_value()
    }
}
