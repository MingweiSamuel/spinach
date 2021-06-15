use std::iter::FromIterator;
use std::cmp::Ordering;

use super::*;

use crate::tag;
use crate::collections::Collection;


pub struct SetUnion<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> Lattice for SetUnion<T> {}

pub trait SetTag<T>: tag::Tag1<T> {}
impl<T> SetTag<T> for tag::HASH_SET {}
impl<T> SetTag<T> for tag::BTREE_SET {}
impl<T> SetTag<T> for tag::VEC {}
impl<T> SetTag<T> for tag::SINGLE {}
impl<T> SetTag<T> for tag::OPTION {}
impl<T, const N: usize> SetTag<T> for tag::ARRAY<N> {}
impl<T, const N: usize> SetTag<T> for tag::MASKED_ARRAY<N> {}

pub struct SetUnionRepr<Tag: SetTag<T>, T> {
    _phantom: std::marker::PhantomData<(Tag, T)>,
}

impl<Tag: SetTag<T>, T> LatticeRepr for SetUnionRepr<Tag, T>
where
    Tag::Bind: Clone,
{
    type Lattice = SetUnion<T>;
    type Repr = Tag::Bind;
}

impl<T, SelfTag: SetTag<T>, DeltaTag: SetTag<T>> Merge<SetUnionRepr<DeltaTag, T>> for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag,  T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<DeltaTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag,  T> as LatticeRepr>::Repr: Collection<T, ()> + Extend<T>,
    <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
{
    fn merge(this: &mut <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr, delta: <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr) -> bool {
        let old_len = this.len();
        this.extend(delta);
        this.len() > old_len
    }
}

impl<T, SelfTag: SetTag<T>, TargetTag: SetTag<T>> Convert<SetUnionRepr<TargetTag, T>> for SetUnionRepr<SelfTag, T>
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

impl<T: 'static, SelfTag: SetTag<T>, TargetTag: SetTag<T>> Compare<SetUnionRepr<TargetTag, T>> for SetUnionRepr<SelfTag, T>
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
    use crate::collections::Single;
    use crate::hide::{Hide, Qualifier, Delta, Value};
    use crate::lattice::mapunion::{MapTag, MapUnionRepr};

    use super::*;
    use super::ord::MaxRepr;

    impl<Tag: SetTag<T>, T> Hide<Value, SetUnionRepr<Tag, T>>
    where
        Tag::Bind: Clone,
        <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: Collection<T, ()>,
    {
        pub fn len(&self) -> Hide<Value, MaxRepr<usize>> {
            Hide::new(self.reveal_ref().len())
        }
    }

    impl<Y: Qualifier, Tag: SetTag<T>, T> Hide<Y, SetUnionRepr<Tag, T>>
    where
        Tag::Bind: Clone,
        <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: Collection<T, ()>,
    {
        pub fn contains(&self, val: &T) -> Hide<Value, MaxRepr<bool>> {
            Hide::new(self.reveal_ref().get(val).is_some())
        }
    }

    impl<Y: Qualifier, T> Hide<Y, SetUnionRepr<tag::SINGLE, T>>
    where
        T: Clone,
    {
        pub fn map_single<U: Clone, F: Fn(T) -> U>(self, f: F) -> Hide<Y, SetUnionRepr<tag::SINGLE, U>> {
            Hide::new(crate::collections::Single((f)(self.into_reveal().0)))
        }

        pub fn filter_map_single<U: Clone, F: Fn(T) -> Option<U>>(self, f: F) -> Hide<Y, SetUnionRepr<tag::OPTION, U>> {
            Hide::new((f)(self.into_reveal().0))
        }
    }

    impl<Y: Qualifier, Tag: SetTag<T>, T> Hide<Y, SetUnionRepr<Tag, T>>
    where
        Tag::Bind: Clone,
        <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    {
        pub fn filter_map<U, TargetTag: SetTag<U>, F: Fn(T) -> Option<U>>(self, f: F) -> Hide<Y, SetUnionRepr<TargetTag, U>>
        where
            SetUnionRepr<TargetTag, U>: LatticeRepr<Lattice = SetUnion<U>>,
            <SetUnionRepr<TargetTag, U> as LatticeRepr>::Repr: FromIterator<U>,
        {
            Hide::new(self.into_reveal().into_iter().filter_map(f).collect())
        }

        pub fn map<U, TargetTag: SetTag<U>, F: Fn(T) -> U>(self, f: F) -> Hide<Y, SetUnionRepr<TargetTag, U>>
        where
            SetUnionRepr<TargetTag, U>: LatticeRepr<Lattice = SetUnion<U>>,
            <SetUnionRepr<TargetTag, U> as LatticeRepr>::Repr: FromIterator<U>,
        {
            Hide::new(self.into_reveal().into_iter().map(f).collect())
        }

        pub fn filter<TargetTag: SetTag<T>, F: Fn(&T) -> bool>(self, f: F) -> Hide<Y, SetUnionRepr<TargetTag, T>>
        where
            SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
            <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: FromIterator<T>,
        {
            Hide::new(self.into_reveal().into_iter().filter(f).collect())
        }

        pub fn flatten<TargetTag: SetTag<T::Item>>(self) -> Hide<Y, SetUnionRepr<TargetTag, T::Item>>
        where
            T: IntoIterator,
            SetUnionRepr<TargetTag, T::Item>: LatticeRepr<Lattice = SetUnion<T::Item>>,
            <SetUnionRepr<TargetTag, T::Item> as LatticeRepr>::Repr: FromIterator<T::Item>,
        {
            Hide::new(self.into_reveal().into_iter().flatten().collect())
        }
    }

    impl<Y: Qualifier, Tag: SetTag<(K, V)>, K: Clone, V: Clone> Hide<Y, SetUnionRepr<Tag, (K, V)>>
    where
        SetUnionRepr<Tag, (K, V)>: LatticeRepr,
        <SetUnionRepr<Tag, (K, V)> as LatticeRepr>::Repr: IntoIterator<Item = (K, V)>,
    {
        pub fn into_map<TargetTag: MapTag<K, Single<V>>>(self) -> Hide<Y, MapUnionRepr<TargetTag, K, SetUnionRepr<tag::SINGLE, V>>>
        where
            TargetTag::Bind: Clone,
            <MapUnionRepr<TargetTag, K, SetUnionRepr<tag::SINGLE, V>> as LatticeRepr>::Repr: FromIterator<(K, Single<V>)>,
        {
            let out = self.into_reveal().into_iter()
                .map(|(k, v)| (k, Single(v)))
                .collect();
            Hide::new(out)
        }
    }

    // impl<Y: Qualifier, Tag: SetTag<(K, V)>, K: Clone, V: Clone> Hide<Y, SetUnionRepr<Tag, (K, V)>>
    // where
    //     SetUnionRepr<Tag, (K, V)>: LatticeRepr,
    //     <SetUnionRepr<Tag, (K, V)> as LatticeRepr>::Repr: IntoIterator<Item = (K, V)>,
    // {
    //     pub fn fold_into_map<TargetTag: MapTag<K, Single<V>>, ValRepr, TargetValRepr>(self) -> Hide<Y, TargetRepr>
    //     where
    //         ValRepr: LatticeRepr<Repr = V>,
    //         TargetTag::Bind: Clone,
    //         MapUnionRepr<TargetTag, K, TargetValRepr>: Merge<MapUnionRepr<tag::SINGLE, K, V>>,
    //         // <MapUnionRepr<TargetTag, K, TargetValRepr> as LatticeRepr>::Repr: Default,

    //         TargetValRepr: LatticeRepr + Merge<ValRepr>,
    //         TargetValRepr::Repr: Default,
    //     {
    //         let mut lattice: <MapUnionRepr<TargetTag, K, TargetValRepr> as LatticeRepr>::Repr = Default::default();
    //         // let mut lattice: TargetValRepr::Repr = Default::default();
    //         for (key, item) in self.into_reveal().into_iter() {
    //             <TargetValRepr as Merge<ValRepr>>::merge(&mut lattice, item);
    //         }
    //         Hide::new(lattice)
    //     }
    // }


    fn __test_things() {
        let my_lattice: Hide<Value, SetUnionRepr<tag::HASH_SET, u32>> =
            Hide::new(vec![ 0, 1, 2, 3, 5, 8, 13 ].into_iter().collect());

        let _: Hide<Value, MaxRepr<usize>> = my_lattice.len();
        let _: Hide<Value, MaxRepr<bool>>  = my_lattice.contains(&4);

        let my_delta: Hide<Delta, SetUnionRepr<tag::HASH_SET, u32>> =
            Hide::new(vec![ 0, 1, 2, 3, 5, 8, 13 ].into_iter().collect());

        // let _: Hide<Value, MaxRepr<usize>> = my_delta.len();
        let _: Hide<Value, MaxRepr<bool>>  = my_delta.contains(&4);
    }
}
