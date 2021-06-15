use std::iter::FromIterator;

use crate::lattice::LatticeRepr;
use crate::lattice::setunion::SetUnion;
use crate::hide::{Hide, Qualifier};

use super::BinaryMorphism;

pub struct Lookup<A, AItem, B, BItem, O>(std::marker::PhantomData<(A, AItem, B, BItem, O)>)
where
    A: LatticeRepr<Lattice = SetUnion<AItem>>,
    B: LatticeRepr<Lattice = SetUnion<BItem>>,
    A::Repr: IntoIterator<Item = AItem>,
    B::Repr: IntoIterator<Item = BItem>,
    AItem: Clone,
    O: LatticeRepr<Lattice = SetUnion<(AItem, BItem)>>,
    O::Repr: FromIterator<(AItem, BItem)>,
;

impl<A, AItem, B, BItem, O> BinaryMorphism for Lookup<A, AItem, B, BItem, O>
where
    A: LatticeRepr<Lattice = SetUnion<AItem>>,
    B: LatticeRepr<Lattice = SetUnion<BItem>>,
    A::Repr: IntoIterator<Item = AItem>,
    B::Repr: IntoIterator<Item = BItem>,
    AItem: Clone,
    O: LatticeRepr<Lattice = SetUnion<(AItem, BItem)>>,
    O::Repr: FromIterator<(AItem, BItem)>,
{
    type InLatReprA = A;
    type InLatReprB = B;
    type OutLatRepr = O;

    fn call<Y: Qualifier>(
        &self,
        item_a: Hide<Y, Self::InLatReprA>, item_b: Hide<Y, Self::InLatReprB>
    )
        -> Hide<Y, Self::OutLatRepr>
    {
        let out = item_a
            .into_reveal()
            .into_iter()
            .flat_map(|item_a| {
                item_b.clone()
                    .into_reveal()
                    .into_iter()
                    .map(move |item_b| (item_a.clone(), item_b))
            })
            .collect();
        Hide::new(out)
    }
}