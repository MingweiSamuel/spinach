use crate::lattice::LatticeRepr;
use crate::hide::{Hide, Qualifier, Delta, Value};

pub trait Monotone {
    type InLatRepr:  LatticeRepr;
    type OutLatRepr: LatticeRepr;
    fn call(&self, item: Hide<Value, Self::InLatRepr>) -> Hide<Value, Self::OutLatRepr>;
}

pub trait Morphism {
    type InLatRepr:  LatticeRepr;
    type OutLatRepr: LatticeRepr;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr>;
}

impl<M: Morphism> Monotone for M {
    type InLatRepr  = <Self as Morphism>::InLatRepr;
    type OutLatRepr = <Self as Morphism>::OutLatRepr;
    fn call(&self, item: Hide<Value, Self::InLatRepr>) -> Hide<Value, Self::OutLatRepr> {
        <Self as Morphism>::call(self, item)
    }
}

pub struct ClosureMorphism<In: LatticeRepr, Out: LatticeRepr, F>
where
    F: Fn(Hide<Delta, In>) -> Hide<Delta, Out>,
{
    func: F,
    _phantom: std::marker::PhantomData<(In, Out)>,
}

impl<In: LatticeRepr, Out: LatticeRepr, F> ClosureMorphism<In, Out, F>
where
    F: Fn(Hide<Delta, In>) -> Hide<Delta, Out>,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<In: LatticeRepr, Out: LatticeRepr, F> Morphism for ClosureMorphism<In, Out, F>
where
    F: Fn(Hide<Delta, In>) -> Hide<Delta, Out>,
{
    type InLatRepr  = In;
    type OutLatRepr = Out;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        (self.func)(item.into_delta()).into_qualifier_reveal()
    }
}
