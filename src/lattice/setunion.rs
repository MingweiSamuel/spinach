use std::iter::FromIterator;
use std::cmp::Ordering;

use super::*;

use crate::tag;
use crate::collections::Collection;


pub struct SetUnion<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> Lattice for SetUnion<T> {}

pub trait SetTag: tag::Tag1 {}
impl SetTag for tag::HASH_SET {}
impl SetTag for tag::BTREE_SET {}
impl SetTag for tag::VEC {}
impl SetTag for tag::SINGLE {}
impl SetTag for tag::OPTION {}
impl<const N: usize> SetTag for tag::ARRAY<N> {}
impl<const N: usize> SetTag for tag::MASKED_ARRAY<N> {}

pub struct SetUnionRepr<Tag: SetTag, T> {
    _phantom: std::marker::PhantomData<(Tag, T)>,
}

impl<Tag: SetTag, T> LatticeRepr for SetUnionRepr<Tag, T> {
    type Lattice = SetUnion<T>;
    type Repr = Tag::Type<T>;
}

impl<T, SelfTag: SetTag, DeltaTag: SetTag> Merge<SetUnionRepr<DeltaTag, T>> for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag,  T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<DeltaTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag,  T> as LatticeRepr>::Repr: Extend<T>,
    <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
{
    fn merge(this: &mut <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr, delta: <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr) {
        this.extend(delta)
    }
}

impl<T, SelfTag: SetTag, TargetTag: SetTag> Convert<SetUnionRepr<TargetTag, T>> for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag,   T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag,   T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: FromIterator<T>,
{
    fn convert(this: <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr) -> <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr {
        this.into_iter().collect()
    }
}

impl<T: 'static, SelfTag: SetTag, TargetTag: SetTag> Compare<SetUnionRepr<TargetTag, T>> for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag,   T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag,   T> as LatticeRepr>::Repr: Collection<T, ()>,
    <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: Collection<T, ()>,
{
    fn compare(this: &<SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr, other: &<SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr) -> Option<Ordering> {
        if this.len() > other.len() {
            if this.keys().all(|key| other.get(key).is_some()) {
                Some(Ordering::Greater)
            }
            else {
                None
            }
        }
        else if this.len() == other.len() {
            if this.keys().all(|key| other.get(key).is_some()) {
                Some(Ordering::Equal)
            }
            else {
                None
            }
        }
        else { // this.len() < other.len()
            if other.keys().all(|key| this.get(key).is_some()) {
                Some(Ordering::Less)
            }
            else {
                None
            }
        }
    }
}

fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    assert_impl_all!(SetUnionRepr<tag::HASH_SET, u32>:
        Merge<SetUnionRepr<tag::HASH_SET, u32>>,
        Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
        Merge<SetUnionRepr<tag::VEC, u32>>,
        Merge<SetUnionRepr<tag::SINGLE, u32>>,
        Merge<SetUnionRepr<tag::OPTION, u32>>,
        Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
        Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
    );

    assert_impl_all!(SetUnionRepr<tag::BTREE_SET, u32>:
        Merge<SetUnionRepr<tag::HASH_SET, u32>>,
        Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
        Merge<SetUnionRepr<tag::VEC, u32>>,
        Merge<SetUnionRepr<tag::SINGLE, u32>>,
        Merge<SetUnionRepr<tag::OPTION, u32>>,
        Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
        Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
    );

    assert_impl_all!(SetUnionRepr<tag::VEC, u32>:
        Merge<SetUnionRepr<tag::HASH_SET, u32>>,
        Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
        Merge<SetUnionRepr<tag::VEC, u32>>,
        Merge<SetUnionRepr<tag::SINGLE, u32>>,
        Merge<SetUnionRepr<tag::OPTION, u32>>,
        Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
        Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
    );

    assert_not_impl_any!(SetUnionRepr<tag::MASKED_ARRAY<8>, u32>:
        Merge<SetUnionRepr<tag::HASH_SET, u32>>,
        Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
        Merge<SetUnionRepr<tag::VEC, u32>>,
        Merge<SetUnionRepr<tag::SINGLE, u32>>,
        Merge<SetUnionRepr<tag::OPTION, u32>>,
        Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
        Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
    );
}