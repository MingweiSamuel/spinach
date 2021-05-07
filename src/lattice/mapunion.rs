use std::cmp::Ordering;

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

pub struct MapUnionRepr<Tag: MapTag, K, B: LatticeRepr> {
    _phantom: std::marker::PhantomData<(Tag, K, B)>,
}

impl<Tag: MapTag, K, B: LatticeRepr> LatticeRepr for MapUnionRepr<Tag, K, B>{
    type Lattice = MapUnion<K, B::Lattice>;
    type Repr = Tag::Type<K, B::Repr>;
}

impl<K: 'static, SelfTag, DeltaTag, SelfLR: LatticeRepr<Lattice = L>, DeltaLR: LatticeRepr<Lattice = L>, L: Lattice> Merge<MapUnionRepr<DeltaTag, K, DeltaLR>> for MapUnionRepr<SelfTag, K, SelfLR>
where
    SelfTag:  MapTag,
    DeltaTag: MapTag,
    MapUnionRepr<SelfTag,  K, SelfLR>:  LatticeRepr<Lattice = MapUnion<K, L>>,
    MapUnionRepr<DeltaTag, K, DeltaLR>: LatticeRepr<Lattice = MapUnion<K, L>>,
    <MapUnionRepr<SelfTag,  K, SelfLR>  as LatticeRepr>::Repr: Extend<(K, SelfLR::Repr)> + Collection<K, SelfLR::Repr>,
    <MapUnionRepr<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr: IntoIterator<Item = (K, DeltaLR::Repr)>,
    SelfLR:  Merge<DeltaLR>,
    DeltaLR: Convert<SelfLR>,
{
    fn merge(this: &mut <MapUnionRepr<SelfTag, K, SelfLR> as LatticeRepr>::Repr, delta: <MapUnionRepr<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr) {
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

impl<K: 'static, SelfTag, DeltaTag, SelfLR: LatticeRepr<Lattice = L>, DeltaLR: LatticeRepr<Lattice = L>, L: Lattice> Compare<MapUnionRepr<DeltaTag, K, DeltaLR>> for MapUnionRepr<SelfTag, K, SelfLR>
where
    SelfTag:  MapTag,
    DeltaTag: MapTag,
    MapUnionRepr<SelfTag,  K, SelfLR>:  LatticeRepr<Lattice = MapUnion<K, L>>,
    MapUnionRepr<DeltaTag, K, DeltaLR>: LatticeRepr<Lattice = MapUnion<K, L>>,
    <MapUnionRepr<SelfTag,  K, SelfLR>  as LatticeRepr>::Repr: Collection<K, SelfLR::Repr>,
    <MapUnionRepr<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr: Collection<K, DeltaLR::Repr>,
    SelfLR: Compare<DeltaLR>,
{
    fn compare(this: &<MapUnionRepr<SelfTag, K, SelfLR> as LatticeRepr>::Repr, other: &<MapUnionRepr<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr) -> Option<Ordering> {
        if this.len() > other.len() {
            for (key, this_value) in this.entries() {
                if let Some(other_value) = other.get(key) {
                    if let Some(Ordering::Less) = SelfLR::compare(this_value, other_value) {
                        return None;
                    }
                }
            }
            Some(Ordering::Greater)
        }
        else if this.len() == other.len() {
            let mut current_ordering = Ordering::Equal;
            for (key, this_value) in this.entries() {
                match other.get(key) {
                    Some(other_value) => {
                        match SelfLR::compare(this_value, other_value) {
                            // current_ordering unchanged
                            Some(Ordering::Equal) => {},
                            // If we get a strict inequality, check if that conflicts with the current_ordering.
                            // Then update the current_ordering.
                            Some(inequal) => {
                                if inequal.reverse() == current_ordering {
                                    // Conflict.
                                    return None;
                                }
                                current_ordering = inequal;
                            }
                            None => return None
                        }
                    }
                    None => {
                        if Ordering::Less == current_ordering {
                            return None;
                        }
                    }
                }
            }
            Some(current_ordering)
        }
        else { // this.len() < other.len()
            for (key, other_value) in other.entries() {
                if let Some(this_value) = this.get(key) {
                    if let Some(Ordering::Greater) = SelfLR::compare(this_value, other_value) {
                        return None;
                    }
                }
            }
            Some(Ordering::Less)
        }
    }
}

fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};
    
    use super::setunion::{SetUnionRepr};

    type HashMapHashSet    = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::HASH_SET, u32>>;
    type HashMapArraySet   = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::ARRAY<8>, u32>>;
    type OptionMapArraySet = MapUnionRepr<tag::OPTION,   String, SetUnionRepr<tag::HASH_SET, u32>>;

    assert_impl_all!(HashMapHashSet: Merge<HashMapHashSet>);
    assert_impl_all!(HashMapHashSet: Merge<HashMapArraySet>);

    assert_not_impl_any!(HashMapArraySet: Merge<HashMapHashSet>);
    assert_not_impl_any!(HashMapArraySet: Merge<HashMapArraySet>);

    assert_not_impl_any!(OptionMapArraySet: Merge<HashMapHashSet>);
    assert_not_impl_any!(OptionMapArraySet: Merge<HashMapArraySet>);
}
