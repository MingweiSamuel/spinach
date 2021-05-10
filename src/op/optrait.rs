use std::task::{Context, Poll};

use crate::hide::{Hide, Delta, Value};
use crate::lattice::LatticeRepr;

pub trait Op {
    /// The output element type of this op.
    type LatRepr: LatticeRepr;
}

pub trait OpDelta: Op {
    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>>;
}

pub trait OpValue: Op {
    fn get_value(&self) -> Hide<Value, Self::LatRepr>;
}
