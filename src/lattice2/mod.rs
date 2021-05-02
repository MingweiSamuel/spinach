use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::iter::Extend;

use set::*;

pub mod set {
    use std::array::IntoIter;
    use std::collections::{BTreeSet, HashSet};
    use std::hash::Hash;

    pub trait Set<T> {
        fn contains(&self, value: &T) -> bool;
    }

    impl<T: Eq + Hash> Set<T> for HashSet<T> {
        fn contains(&self, value: &T) -> bool {
            self.contains(value)
        }
    }

    impl<T: Eq + Ord> Set<T> for BTreeSet<T> {
        fn contains(&self, value: &T) -> bool {
            self.contains(value)
        }
    }

    impl<T: Eq> Set<T> for Vec<T> {
        fn contains(&self, value: &T) -> bool {
            <[T]>::contains(self, value)
        }
    }

    impl<T: Eq, const N: usize> Set<T> for [T; N] {
        fn contains(&self, value: &T) -> bool {
            self.iter().any(|item| value == item)
        }
    }


    #[repr(transparent)]
    pub struct Single<T>(pub T);
    impl<T> IntoIterator for Single<T> {
        type Item = T;
        type IntoIter = <Option<T> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            Some(self.0).into_iter()
        }
    }


    pub struct MaskedArray<T, const N: usize> {
        pub mask: [bool; N],
        pub vals: [T; N],
    }
    impl<T, const N: usize> IntoIterator for MaskedArray<T, N> {
        type Item = T;
        type IntoIter = impl Iterator<Item = Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            IntoIter::new(self.mask)
                .zip(IntoIter::new(self.vals))
                .filter(|(mask, _)| *mask)
                .map(|(_, val)| val)
        }
    }


    pub struct Array<T, const N: usize>(pub [T; N]);
    impl<T, const N: usize> IntoIterator for Array<T, N> {
        type Item = T;
        type IntoIter = IntoIter<T, N>;

        fn into_iter(self) -> Self::IntoIter {
            IntoIter::new(self.0)
        }
    }
}



pub struct LatticePoint<L: Lattice, R: Repr<L>> {
    pub /* TODO!! */ value: R,
    _phantom: std::marker::PhantomData<L>,
}



pub trait Lattice {}

pub trait Repr<L: Lattice> {}

pub trait Merge<L: Lattice, Other>: Repr<L> {
    fn merge(&mut self, value: Other);
}

pub trait ReprOrd<L: Lattice, Other>: Repr<L> {
    fn partial_cmp(&self, other: &Other) -> Option<Ordering>;
}

pub trait ReprCollection<L: Lattice, T>: Repr<L> {
    type MinDelta: Repr<L>;
    fn min_delta<R: Repr<L> + Set<T>>(self, existing: &R) -> Option<Self::MinDelta>;

    type Map<U>: Repr<L>;
    fn map<U>(self, f: impl Fn(T) -> U) -> Self::Map<U>;

    type Filter: Repr<L>;
    fn filter(self, f: impl Fn(&T) -> bool) -> Self::Filter;

    // type Flatten<U>;
    // fn flatten<U>(self, f: impl Fn(T) -> U) -> Self::Flatten<U>;
}

pub enum SetUnion {}
impl Lattice for SetUnion {}

impl<T> Repr<SetUnion> for HashSet<T> {}
impl<T> Repr<SetUnion> for BTreeSet<T> {}
impl<T> Repr<SetUnion> for Vec<T> {}
impl<T> Repr<SetUnion> for Single<T> {}
impl<T> Repr<SetUnion> for Option<T> {}
impl<T, const N: usize> Repr<SetUnion> for Array<T, N> {}
impl<T, const N: usize> Repr<SetUnion> for MaskedArray<T, N> {}

impl<T: Eq, E: Extend<T> + Repr<SetUnion>, I: IntoIterator<Item=T>> Merge<SetUnion, I> for E {
    fn merge(&mut self, value: I) {
        self.extend(value)
    }
}


pub enum Max {}
impl Lattice for Max {}

impl<T: Ord> Repr<Max> for T {}
impl<T: Ord, U: Into<T>> Merge<Max, U> for T {
    fn merge(&mut self, value: U) {
        let value = value.into();
        if value > *self { *self = value; }
    }
}



pub enum Min {}
impl Lattice for Min {}

impl<T: Ord> Repr<Min> for T {}
impl<T: Ord, U: Into<T>> Merge<Min, U> for T {
    fn merge(&mut self, value: U) {
        let value = value.into();
        if value < *self { *self = value; }
    }
}

impl<T: Eq + Hash> ReprOrd<SetUnion, HashSet<T>> for HashSet<T> {
    fn partial_cmp(&self, other: &HashSet<T>) -> Option<Ordering> {
        let s = self.union(other).count();
        if s != self.len() && s != other.len() {
            None
        } else if s == self.len() {
            if s == other.len() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            Some(Ordering::Less)
        }
    }
}

impl<T: Eq + Ord> ReprOrd<SetUnion, BTreeSet<T>> for BTreeSet<T> {
    fn partial_cmp(&self, other: &BTreeSet<T>) -> Option<Ordering> {
        let s = self.union(other).count();
        if s != self.len() && s != other.len() {
            None
        } else if s == self.len() {
            if s == other.len() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            Some(Ordering::Less)
        }
    }
}

impl<T: Eq + Hash> ReprCollection<SetUnion, T> for HashSet<T> {
    type MinDelta = HashSet<T>;
    fn min_delta<R: Repr<SetUnion> + Set<T>>(mut self, existing: &R) -> Option<Self::MinDelta> {
        self.retain(|x| !existing.contains(x));
        if self.is_empty() { None } else { Some(self) }
    }

    type Map<U> = Vec<U>;
    fn map<U>(self, f: impl Fn(T) -> U) -> Self::Map<U> {
        self.into_iter().map(f).collect()
    }

    type Filter = Vec<T>;
    fn filter(self, f: impl Fn(&T) -> bool) -> Self::Filter {
        self.into_iter().filter(f).collect()
    }
}

impl<T: Eq + Ord> ReprCollection<SetUnion, T> for BTreeSet<T> {
    type MinDelta = BTreeSet<T>;
    fn min_delta<R: Repr<SetUnion> + Set<T>>(mut self, existing: &R) -> Option<Self::MinDelta> {
        self.retain(|x| !existing.contains(x));
        if self.is_empty() { None } else { Some(self) }
    }

    type Map<U> = Vec<U>;
    fn map<U>(self, f: impl Fn(T) -> U) -> Self::Map<U> {
        self.into_iter().map(f).collect()
    }

    type Filter = Vec<T>;
    fn filter(self, f: impl Fn(&T) -> bool) -> Self::Filter {
        self.into_iter().filter(f).collect()
    }
}

impl<T: Eq> ReprCollection<SetUnion, T> for Vec<T> {
    type MinDelta = Vec<T>;
    fn min_delta<R: Repr<SetUnion> + Set<T>>(mut self, existing: &R) -> Option<Self::MinDelta> {
        self.retain(|x| !existing.contains(x));
        if self.is_empty() { None } else { Some(self) }
    }

    type Map<U> = Vec<U>;
    fn map<U>(self, f: impl Fn(T) -> U) -> Self::Map<U> {
        self.into_iter().map(f).collect()
    }

    type Filter = Vec<T>;
    fn filter(self, f: impl Fn(&T) -> bool) -> Self::Filter {
        self.into_iter().filter(f).collect()
    }
}

impl<T: Eq, const N: usize> ReprCollection<SetUnion, T> for Array<T, N> {
    type MinDelta = MaskedArray<T, N>;
    fn min_delta<R: Repr<SetUnion> + Set<T>>(self, existing: &R) -> Option<Self::MinDelta> {
        let mut any_new = false;
        let out = MaskedArray {
            mask: self.0.each_ref().map(|item| {
                let is_new = !existing.contains(item);
                any_new |= is_new;
                is_new
            }),
            vals: self.0,
        };
        if any_new { Some(out) } else { None }
    }

    type Map<U> = Array<U, N>;
    fn map<U>(self, f: impl Fn(T) -> U) -> Self::Map<U> {
        Array(self.0.map(f))
    }

    type Filter = MaskedArray<T, N>;
    fn filter(self, f: impl Fn(&T) -> bool) -> Self::Filter {
        MaskedArray {
            mask: self.0.each_ref().map(f),
            vals: self.0,
        }
    }
}

#[test]
pub fn test_setunion_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    assert_impl_all!(HashSet<u32>: Merge<SetUnion, HashSet<u32>>);
    assert_impl_all!(HashSet<u32>: Merge<SetUnion, BTreeSet<u32>>);
    assert_impl_all!(HashSet<u32>: Merge<SetUnion, Vec<u32>>);
    assert_impl_all!(HashSet<u32>: Merge<SetUnion, Single<u32>>);
    assert_impl_all!(HashSet<u32>: Merge<SetUnion, Option<u32>>);
    assert_impl_all!(HashSet<u32>: Merge<SetUnion, Array<u32, 8>>);
    assert_impl_all!(HashSet<u32>: Merge<SetUnion, MaskedArray<u32, 8>>);

    assert_impl_all!(BTreeSet<u32>: Merge<SetUnion, HashSet<u32>>);
    assert_impl_all!(BTreeSet<u32>: Merge<SetUnion, BTreeSet<u32>>);
    assert_impl_all!(BTreeSet<u32>: Merge<SetUnion, Vec<u32>>);
    assert_impl_all!(BTreeSet<u32>: Merge<SetUnion, Single<u32>>);
    assert_impl_all!(BTreeSet<u32>: Merge<SetUnion, Option<u32>>);
    assert_impl_all!(BTreeSet<u32>: Merge<SetUnion, Array<u32, 8>>);
    assert_impl_all!(BTreeSet<u32>: Merge<SetUnion, MaskedArray<u32, 8>>);

    assert_impl_all!(Vec<u32>: Merge<SetUnion, HashSet<u32>>);
    assert_impl_all!(Vec<u32>: Merge<SetUnion, BTreeSet<u32>>);
    assert_impl_all!(Vec<u32>: Merge<SetUnion, Vec<u32>>);
    assert_impl_all!(Vec<u32>: Merge<SetUnion, Single<u32>>);
    assert_impl_all!(Vec<u32>: Merge<SetUnion, Option<u32>>);
    assert_impl_all!(Vec<u32>: Merge<SetUnion, Array<u32, 8>>);
    assert_impl_all!(Vec<u32>: Merge<SetUnion, MaskedArray<u32, 8>>);

    assert_not_impl_any!(Single<u32>: Merge<SetUnion, HashSet<u32>>);
    assert_not_impl_any!(Single<u32>: Merge<SetUnion, BTreeSet<u32>>);
    assert_not_impl_any!(Single<u32>: Merge<SetUnion, Vec<u32>>);
    assert_not_impl_any!(Single<u32>: Merge<SetUnion, Single<u32>>);
    assert_not_impl_any!(Single<u32>: Merge<SetUnion, Option<u32>>);
    assert_not_impl_any!(Single<u32>: Merge<SetUnion, Array<u32, 8>>);
    assert_not_impl_any!(Single<u32>: Merge<SetUnion, MaskedArray<u32, 8>>);

    assert_not_impl_any!(Option<u32>: Merge<SetUnion, HashSet<u32>>);
    assert_not_impl_any!(Option<u32>: Merge<SetUnion, BTreeSet<u32>>);
    assert_not_impl_any!(Option<u32>: Merge<SetUnion, Vec<u32>>);
    assert_not_impl_any!(Option<u32>: Merge<SetUnion, Single<u32>>);
    assert_not_impl_any!(Option<u32>: Merge<SetUnion, Option<u32>>);
    assert_not_impl_any!(Option<u32>: Merge<SetUnion, Array<u32, 8>>);
    assert_not_impl_any!(Option<u32>: Merge<SetUnion, MaskedArray<u32, 8>>);

    assert_not_impl_any!(Array<u32, 8>: Merge<SetUnion, HashSet<u32>>);
    assert_not_impl_any!(Array<u32, 8>: Merge<SetUnion, BTreeSet<u32>>);
    assert_not_impl_any!(Array<u32, 8>: Merge<SetUnion, Vec<u32>>);
    assert_not_impl_any!(Array<u32, 8>: Merge<SetUnion, Single<u32>>);
    assert_not_impl_any!(Array<u32, 8>: Merge<SetUnion, Option<u32>>);
    assert_not_impl_any!(Array<u32, 8>: Merge<SetUnion, Array<u32, 8>>);
    assert_not_impl_any!(Array<u32, 8>: Merge<SetUnion, MaskedArray<u32, 8>>);

    assert_not_impl_any!(MaskedArray<u32, 8>: Merge<SetUnion, HashSet<u32>>);
    assert_not_impl_any!(MaskedArray<u32, 8>: Merge<SetUnion, BTreeSet<u32>>);
    assert_not_impl_any!(MaskedArray<u32, 8>: Merge<SetUnion, Vec<u32>>);
    assert_not_impl_any!(MaskedArray<u32, 8>: Merge<SetUnion, Single<u32>>);
    assert_not_impl_any!(MaskedArray<u32, 8>: Merge<SetUnion, Option<u32>>);
    assert_not_impl_any!(MaskedArray<u32, 8>: Merge<SetUnion, Array<u32, 8>>);
    assert_not_impl_any!(MaskedArray<u32, 8>: Merge<SetUnion, MaskedArray<u32, 8>>);
}

#[test]
pub fn test_ord_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    assert_impl_all!(u32: Merge<Max, u32>);
    assert_not_impl_any!(u32: Merge<Max, u64>);

    assert_impl_all!(u64: Merge<Max, u32>);
    assert_impl_all!(u64: Merge<Max, u64>);

    assert_impl_all!(u32: Merge<Min, u32>);
    assert_not_impl_any!(u32: Merge<Min, u64>);

    assert_impl_all!(u64: Merge<Min, u32>);
    assert_impl_all!(u64: Merge<Min, u64>);
}
