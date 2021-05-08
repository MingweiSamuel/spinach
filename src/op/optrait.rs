use std::task::{Context, Poll};

use crate::hide::{Hide, Delta, Value};
use crate::lattice::LatticeRepr;

pub trait Op<'s> {
    /// The output element type of this op.
    type LatRepr: LatticeRepr;
}

pub trait OpDelta<'s>: Op<'s> {
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>>;
}

pub trait OpValue<'s>: Op<'s> {
    fn get_value(&'s self) -> Hide<Value, Self::LatRepr>;
}
