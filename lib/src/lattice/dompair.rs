use std::cmp::Ordering;

use super::*;

use crate::tag;

pub struct DomPair<LA: Lattice, LB: Lattice> {
    _phantom: std::marker::PhantomData<(LA, LB)>,
}
impl<LA: Lattice, LB: Lattice> Lattice for DomPair<LA, LB> {}

pub struct DomPairRepr<RA: LatticeRepr, RB: LatticeRepr> {
    _phantom: std::marker::PhantomData<(RA, RB)>,
}
impl<RA: LatticeRepr, RB: LatticeRepr> LatticeRepr for DomPairRepr<RA, RB> {
    type Lattice = DomPair<RA::Lattice, RB::Lattice>;
    type Repr = (RA::Repr, RB::Repr);
}

impl<SelfRA, SelfRB, DeltaRA, DeltaRB, LA, LB> Merge<DomPairRepr<DeltaRA, DeltaRB>> for DomPairRepr<SelfRA, SelfRB>
where
    LA: Lattice,
    LB: Lattice,
    SelfRA:  LatticeRepr<Lattice = LA>,
    SelfRB:  LatticeRepr<Lattice = LB>,
    DeltaRA: LatticeRepr<Lattice = LA>,
    DeltaRB: LatticeRepr<Lattice = LB>,
    SelfRA:  Merge<DeltaRA> + Compare<DeltaRA>,
    SelfRB:  Merge<DeltaRB> + Compare<DeltaRB>,
    DeltaRA: Convert<SelfRA>,
    DeltaRB: Convert<SelfRB>,
{
    fn merge(this: &mut <DomPairRepr<SelfRA, SelfRB> as LatticeRepr>::Repr, delta: <DomPairRepr<DeltaRA, DeltaRB> as LatticeRepr>::Repr) -> bool {
        match SelfRA::compare(&this.0, &delta.0) {
            None => {
                SelfRA::merge(&mut this.0, delta.0);
                SelfRB::merge(&mut this.1, delta.1);
                true
            }
            Some(Ordering::Equal) => {
                SelfRB::merge(&mut this.1, delta.1)
            }
            Some(Ordering::Less) => {
                *this = (
                    DeltaRA::convert(delta.0),
                    DeltaRB::convert(delta.1),
                );
                true
            }
            Some(Ordering::Greater) => false
        }
    }
}


impl<SelfRA, SelfRB, DeltaRA, DeltaRB, LA, LB> Compare<DomPairRepr<DeltaRA, DeltaRB>> for DomPairRepr<SelfRA, SelfRB>
where
    LA: Lattice,
    LB: Lattice,
    SelfRA:  LatticeRepr<Lattice = LA>,
    SelfRB:  LatticeRepr<Lattice = LB>,
    DeltaRA: LatticeRepr<Lattice = LA>,
    DeltaRB: LatticeRepr<Lattice = LB>,
    SelfRA:  Compare<DeltaRA>,
    SelfRB:  Compare<DeltaRB>,
{
    fn compare(this: &<DomPairRepr<SelfRA, SelfRB> as LatticeRepr>::Repr, other: &<DomPairRepr<DeltaRA, DeltaRB> as LatticeRepr>::Repr) -> Option<Ordering> {
        SelfRA::compare(&this.0, &other.0)
            .or_else(|| SelfRB::compare(&this.1, &other.1))
    }
}

fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};
    
    use super::setunion::{SetUnionRepr};

    type HashSetHashSet   = DomPairRepr<SetUnionRepr<tag::HASH_SET, u32>, SetUnionRepr<tag::HASH_SET, u32>>;
    type HashSetArraySet  = DomPairRepr<SetUnionRepr<tag::HASH_SET, u32>, SetUnionRepr<tag::ARRAY<8>, u32>>;
    type ArraySetHashSet  = DomPairRepr<SetUnionRepr<tag::ARRAY<8>, u32>, SetUnionRepr<tag::HASH_SET, u32>>;
    type ArraySetArraySet = DomPairRepr<SetUnionRepr<tag::ARRAY<8>, u32>, SetUnionRepr<tag::ARRAY<8>, u32>>;

    assert_impl_all!(HashSetHashSet:
        Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );

    assert_not_impl_any!(HashSetArraySet:
        Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );

    assert_not_impl_any!(ArraySetHashSet:
        Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );

    assert_not_impl_any!(ArraySetArraySet:
        Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );
}
