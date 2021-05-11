use std::task::{Context, Poll};

use crate::hide::{Hide, Delta, Value};
use crate::lattice::{LatticeRepr, Merge, Convert};

use super::*;

pub struct MergeOp<A: Op, B: Op>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
{
    op_a: A,
    op_b: B,
}

impl<A: Op, B: Op> MergeOp<A, B>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
{
    pub fn new(op_a: A, op_b: B) -> Self {
        Self { op_a, op_b }
    }
}

impl<A: Op, B: Op> Op for MergeOp<A, B>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
{
    type LatRepr = A::LatRepr;
}

impl<A: OpDelta, B: OpDelta> OpDelta for MergeOp<A, B>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
    B::LatRepr: Convert<A::LatRepr>,
{
    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        let not_ready = match self.op_a.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => return Poll::Ready(Some(delta)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        };
        match self.op_b.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => Poll::Ready(Some(Hide::new(<B::LatRepr as Convert<A::LatRepr>>::convert(delta.into_reveal())))),
            Poll::Ready(None) => not_ready,
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<A: OpValue, B: OpValue> OpValue for MergeOp<A, B>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
    A::LatRepr: Merge<B::LatRepr>,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        let mut val = self.op_a.get_value().into_reveal();
        <A::LatRepr as Merge<B::LatRepr>>::merge(&mut val, self.op_b.get_value().into_reveal());
        Hide::new(val)
    }
}
