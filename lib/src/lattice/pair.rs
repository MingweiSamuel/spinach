use std::cmp::Ordering;

use super::*;

use crate::tag;

pub struct Pair<La: Lattice, Lb: Lattice> {
    _phantom: std::marker::PhantomData<(La, Lb)>,
}
impl<La: Lattice, Lb: Lattice> Lattice for Pair<La, Lb> {}

pub struct PairRepr<Ra: LatticeRepr, Rb: LatticeRepr> {
    _phantom: std::marker::PhantomData<(Ra, Rb)>,
}
impl<Ra: LatticeRepr, Rb: LatticeRepr> LatticeRepr for PairRepr<Ra, Rb> {
    type Lattice = Pair<Ra::Lattice, Rb::Lattice>;
    type Repr = (Ra::Repr, Rb::Repr);
}

impl<SelfRA, SelfRB, DeltaRA, DeltaRB, La, Lb> Merge<PairRepr<DeltaRA, DeltaRB>> for PairRepr<SelfRA, SelfRB>
where
    La: Lattice,
    Lb: Lattice,
    SelfRA:  LatticeRepr<Lattice = La>,
    SelfRB:  LatticeRepr<Lattice = Lb>,
    DeltaRA: LatticeRepr<Lattice = La>,
    DeltaRB: LatticeRepr<Lattice = Lb>,
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


impl<SelfRA, SelfRB, DeltaRA, DeltaRB, La, Lb> Compare<PairRepr<DeltaRA, DeltaRB>> for PairRepr<SelfRA, SelfRB>
where
    La: Lattice,
    Lb: Lattice,
    SelfRA:  LatticeRepr<Lattice = La>,
    SelfRB:  LatticeRepr<Lattice = Lb>,
    DeltaRA: LatticeRepr<Lattice = La>,
    DeltaRB: LatticeRepr<Lattice = Lb>,
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

    impl<Y: Qualifier, Ra: LatticeRepr, Rb: LatticeRepr> Hide<Y, PairRepr<Ra, Rb>> {
        pub fn split(self) -> (Hide<Y, Ra>, Hide<Y, Rb>) {
            let (a, b) = self.into_reveal();
            (Hide::new(a), Hide::new(b))
        }

        pub fn zip(a: Hide<Y, Ra>, b: Hide<Y, Rb>) -> Self {
            Hide::new((a.into_reveal(), b.into_reveal()))
        }
    }

    impl<Y: Qualifier, Ra: LatticeRepr, Rb: LatticeRepr> Hide<Y, PairRepr<Ra, Rb>>
    where
        Ra::Repr: IntoIterator,
        Rb::Repr: Clone,
    {
        pub fn partial_cartesian_product<TargetTag>(self) -> Hide<Y, SetUnionRepr<TargetTag, (<Ra::Repr as IntoIterator>::Item, Rb::Repr)>>
        where
            TargetTag: SetTag<(<Ra::Repr as IntoIterator>::Item, Rb::Repr)>,
            SetUnionRepr<TargetTag, (<Ra::Repr as IntoIterator>::Item, Rb::Repr)>: LatticeRepr,
            <SetUnionRepr<TargetTag, (<Ra::Repr as IntoIterator>::Item, Rb::Repr)> as LatticeRepr>::Repr: Clone + FromIterator<(<Ra::Repr as IntoIterator>::Item, Rb::Repr)>,
        {
            let (a, b) = self.into_reveal();
            let out = a.into_iter()
                .map(|item_a| (item_a, b.clone()))
                .collect();
            Hide::new(out)
        }
    }
}
