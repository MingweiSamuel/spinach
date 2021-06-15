use std::cmp::Ordering;

use super::*;

use crate::tag;

pub struct Pair<LA: Lattice, LB: Lattice> {
    _phantom: std::marker::PhantomData<(LA, LB)>,
}
impl<LA: Lattice, LB: Lattice> Lattice for Pair<LA, LB> {}

pub struct PairRepr<RA: LatticeRepr, RB: LatticeRepr> {
    _phantom: std::marker::PhantomData<(RA, RB)>,
}
impl<RA: LatticeRepr, RB: LatticeRepr> LatticeRepr for PairRepr<RA, RB> {
    type Lattice = Pair<RA::Lattice, RB::Lattice>;
    type Repr = (RA::Repr, RB::Repr);
}

impl<SelfRA, SelfRB, DeltaRA, DeltaRB, LA, LB> Merge<PairRepr<DeltaRA, DeltaRB>> for PairRepr<SelfRA, SelfRB>
where
    LA: Lattice,
    LB: Lattice,
    SelfRA:  LatticeRepr<Lattice = LA>,
    SelfRB:  LatticeRepr<Lattice = LB>,
    DeltaRA: LatticeRepr<Lattice = LA>,
    DeltaRB: LatticeRepr<Lattice = LB>,
    SelfRA:  Merge<DeltaRA>,
    SelfRB:  Merge<DeltaRB>,
    DeltaRA: Convert<SelfRA>,
    DeltaRB: Convert<SelfRB>,
{
    fn merge(this: &mut <PairRepr<SelfRA, SelfRB> as LatticeRepr>::Repr, delta: <PairRepr<DeltaRA, DeltaRB> as LatticeRepr>::Repr) -> bool {
        // Do NOT use short-circuiting `&&`.
        SelfRA::merge(&mut this.0, delta.0) & SelfRB::merge(&mut this.1, delta.1)
    }
}


impl<SelfRA, SelfRB, DeltaRA, DeltaRB, LA, LB> Compare<PairRepr<DeltaRA, DeltaRB>> for PairRepr<SelfRA, SelfRB>
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
    fn compare(this: &<PairRepr<SelfRA, SelfRB> as LatticeRepr>::Repr, other: &<PairRepr<DeltaRA, DeltaRB> as LatticeRepr>::Repr) -> Option<Ordering> {
        let ord_a = SelfRA::compare(&this.0, &other.0);
        let ord_b = SelfRB::compare(&this.1, &other.1);
        if ord_a == ord_b {
            ord_a
        }
        else {
            None
        }
    }
}

fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    use super::setunion::{SetUnionRepr};

    type HashSetHashSet   = PairRepr<SetUnionRepr<tag::HASH_SET, u32>, SetUnionRepr<tag::HASH_SET, u32>>;
    type HashSetArraySet  = PairRepr<SetUnionRepr<tag::HASH_SET, u32>, SetUnionRepr<tag::ARRAY<8>, u32>>;
    type ArraySetHashSet  = PairRepr<SetUnionRepr<tag::ARRAY<8>, u32>, SetUnionRepr<tag::HASH_SET, u32>>;
    type ArraySetArraySet = PairRepr<SetUnionRepr<tag::ARRAY<8>, u32>, SetUnionRepr<tag::ARRAY<8>, u32>>;

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

mod fns {
    use std::iter::FromIterator;

    use crate::hide::{Hide, Qualifier};
    use crate::lattice::setunion::{SetUnionRepr, SetTag};

    use super::*;

    impl<Y: Qualifier, LA: LatticeRepr, LB: LatticeRepr> Hide<Y, PairRepr<LA, LB>>
    where
        LA::Repr: IntoIterator,
        LB::Repr: Clone,
    {
        pub fn partial_cartesian_product<TargetTag>(self) -> Hide<Y, SetUnionRepr<TargetTag, (<LA::Repr as IntoIterator>::Item, LB::Repr)>>
        where
            TargetTag: SetTag<(<LA::Repr as IntoIterator>::Item, LB::Repr)>,
            TargetTag::Bind: Clone,
            <SetUnionRepr<TargetTag, (<LA::Repr as IntoIterator>::Item, LB::Repr)> as LatticeRepr>::Repr: FromIterator<(<LA::Repr as IntoIterator>::Item, LB::Repr)>,
        {
            let (a, b) = self.into_reveal();
            let out = a.into_iter()
                .map(|item_a| (item_a, b.clone()))
                .collect();
            Hide::new(out)
        }
    }
}
