use super::*;

use crate::collections::*;
use crate::tag;

pub struct MapUnion<K, L: Lattice> {
    _phantom: std::marker::PhantomData<(K, L)>,
}
impl<K, L: Lattice> Lattice for MapUnion<K, L> {}

pub trait MapTag: tag::Tag2 {}
impl MapTag for tag::HASH_MAP {}
impl MapTag for tag::BTREE_MAP {}
impl MapTag for tag::VEC {}
impl MapTag for tag::SINGLE {}
impl MapTag for tag::OPTION {}
impl<const N: usize> MapTag for tag::ARRAY<N> {}
impl<const N: usize> MapTag for tag::MASKED_ARRAY<N> {}

pub struct MapUnionBind<Tag: MapTag, K, B: LatticeRepr> {
    _phantom: std::marker::PhantomData<(Tag, K, B)>,
}

impl<Tag: MapTag, K, B: LatticeRepr> LatticeRepr for MapUnionBind<Tag, K, B>{
    type Lattice = MapUnion<K, B::Lattice>;
    type Repr = Tag::Type<K, B::Repr>;
}

impl<K, SelfTag, DeltaTag, SelfLR: LatticeRepr<Lattice = L>, DeltaLR: LatticeRepr<Lattice = L>, L: Lattice> Merge<MapUnionBind<DeltaTag, K, DeltaLR>> for MapUnionBind<SelfTag, K, SelfLR>
where
    SelfTag:  MapTag,
    DeltaTag: MapTag,
    MapUnionBind<SelfTag,  K, SelfLR>: LatticeRepr<Lattice = MapUnion<K, L>>,
    MapUnionBind<DeltaTag, K, DeltaLR>: LatticeRepr<Lattice = MapUnion<K, L>>,
    <MapUnionBind<SelfTag,  K, SelfLR> as LatticeRepr>::Repr: Extend<(K, SelfLR::Repr)> + Dict<K, SelfLR::Repr>,
    <MapUnionBind<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr: IntoIterator<Item = (K, DeltaLR::Repr)>,
    SelfLR:  Merge<DeltaLR>,
    DeltaLR: Convert<SelfLR>,
{
    fn merge(this: &mut <MapUnionBind<SelfTag, K, SelfLR> as LatticeRepr>::Repr, delta: <MapUnionBind<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr) {
        let iter: Vec<(K, SelfLR::Repr)> = delta.into_iter()
            .filter_map(|(k, v)| {
                match this.get_mut(&k) {
                    Some(target_val) => {
                        <SelfLR as Merge<DeltaLR>>::merge(target_val, v);
                        None
                    }
                    None => {
                        let val: SelfLR::Repr = <DeltaLR as Convert<SelfLR>>::convert(v);
                        Some((k, val))
                    }
                }
            })
            .collect();
        this.extend(iter);
    }
}

fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};
    
    use super::setunion::{SetUnionRepr};

    type HashMapHashSet    = MapUnionBind<tag::HASH_MAP, String, SetUnionRepr<tag::HASH_SET, u32>>;
    type HashMapArraySet   = MapUnionBind<tag::HASH_MAP, String, SetUnionRepr<tag::ARRAY<8>, u32>>;
    type OptionMapArraySet = MapUnionBind<tag::OPTION,   String, SetUnionRepr<tag::HASH_SET, u32>>;

    assert_impl_all!(HashMapHashSet: Merge<HashMapHashSet>);
    assert_impl_all!(HashMapHashSet: Merge<HashMapArraySet>);

    assert_not_impl_any!(HashMapArraySet: Merge<HashMapHashSet>);
    assert_not_impl_any!(HashMapArraySet: Merge<HashMapArraySet>);

    assert_not_impl_any!(OptionMapArraySet: Merge<HashMapHashSet>);
    assert_not_impl_any!(OptionMapArraySet: Merge<HashMapArraySet>);
}