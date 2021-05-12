//! Function traits.

use crate::lattice::LatticeRepr;
use crate::hide::{Hide, Type, Value};

pub trait Morphism {
    type InLatRepr: LatticeRepr;
    type OutLatRepr: LatticeRepr;
    fn call<Y: Type>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr>;
}

pub trait Monotone {
    type InLatRepr: LatticeRepr;
    type OutLatRepr: LatticeRepr;
    fn call(&self, item: Hide<Value, Self::InLatRepr>) -> Hide<Value, Self::OutLatRepr>;
}
