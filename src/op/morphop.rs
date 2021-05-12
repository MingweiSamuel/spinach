use std::task::{Context, Poll};

use crate::hide::{Hide, Delta, Value};
use crate::func::Morphism;

use super::*;

pub struct MorphismOp<O: Op, F: Morphism<InLatRepr = O::LatRepr>> {
    op: O,
    f: F,
}

impl<O: Op, F: Morphism<InLatRepr = O::LatRepr>> MorphismOp<O, F> {
    pub fn new(op: O, f: F) -> Self {
        Self { op, f }
    }
}

impl<O: Op, F: Morphism<InLatRepr = O::LatRepr>> Op for MorphismOp<O, F> {
    type LatRepr = F::OutLatRepr;
}

impl<O: OpDelta, F: Morphism<InLatRepr = O::LatRepr>> OpDelta for MorphismOp<O, F> {
    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => Poll::Ready(Some(self.f.call(delta))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<O: OpValue, F: Morphism<InLatRepr = O::LatRepr>> OpValue for MorphismOp<O, F> {
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        self.f.call(self.op.get_value())
    }
}
