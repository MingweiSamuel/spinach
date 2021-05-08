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
    type Repr = Tag::Bind<T>;
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

mod fns {
    use crate::hide::{Hide, Delta, Value};

    use super::*;
    use super::ord::MaxRepr;

    impl<Tag: SetTag, T> Hide<Value, SetUnionRepr<Tag, T>>
    where
        <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: Collection<T, ()>,
    {
        pub fn len(&self) -> Hide<Value, MaxRepr<usize>> {
            Hide::new(self.as_reveal().len())
        }
    }

    impl<Tag: SetTag, T> Hide<Delta, SetUnionRepr<Tag, T>>
    where
        <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: Collection<T, ()>,
    {
        pub fn contains(&self, val: &T) -> Hide<Value, MaxRepr<bool>> {
            Hide::new(self.as_reveal().get(val).is_some())
        }
    }

    // impl<Tag: SetTag, T> Hide<Delta, SetUnionRepr<Tag, T>>
    // where
    //     <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    // {
    //     pub fn map<U, TargetTag: SetTag>(self, f: impl Fn(T) -> U) -> Hide<Delta, SetUnionRepr<TargetTag, U>>
    //     where
    //         SetUnionRepr<TargetTag, U>: LatticeRepr<Lattice = SetUnion<U>>,
    //         <SetUnionRepr<TargetTag, U> as LatticeRepr>::Repr: FromIterator<U>,
    //     {
    //         Hide::new(self.into_reveal().into_iter().map(f).collect())
    //     }

    //     pub fn filter<TargetTag: SetTag>(self, f: impl Fn(&T) -> bool) -> Hide<Delta, SetUnionRepr<TargetTag, T>>
    //     where
    //         SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    //         <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: FromIterator<T>,
    //     {
    //         Hide::new(self.into_reveal().into_iter().filter(f).collect())
    //     }

    //     pub fn flatten<TargetTag: SetTag>(self) -> Hide<Delta, SetUnionRepr<TargetTag, T::Item>>
    //     where
    //         T: IntoIterator,
    //         SetUnionRepr<TargetTag, T::Item>: LatticeRepr<Lattice = SetUnion<T::Item>>,
    //         <SetUnionRepr<TargetTag, T::Item> as LatticeRepr>::Repr: FromIterator<T::Item>,
    //     {
    //         Hide::new(self.into_reveal().into_iter().flatten().collect())
    //     }
    // }


    fn __test_things() {
        let my_lattice: Hide<Value, SetUnionRepr<tag::HASH_SET, u32>> =
            Hide::new(vec![ 0, 1, 2, 3, 5, 8, 13 ].into_iter().collect());

        let _: Hide<Value, MaxRepr<usize>> = my_lattice.len();
        let _: Hide<Value, MaxRepr<bool>>  = my_lattice.contains(&4);

        let my_delta: &Hide<Delta, SetUnionRepr<tag::HASH_SET, u32>> = &my_lattice;

        // let _: Hide<Value, MaxRepr<usize>> = my_delta.len();
        let _: Hide<Value, MaxRepr<bool>>  = my_delta.contains(&4);
    }
}
