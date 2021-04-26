use std::collections::{btree_set, hash_set, BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::ops::Neg;

use ref_cast::RefCast;

use super::{Lattice, Max, Min, Union, MapUnion, RefOptional};

#[repr(transparent)]
#[derive(RefCast)]
pub struct Hide<F: Lattice>(F::Domain);

impl<F: Lattice> Hide<F> {
    pub fn from_val(val: F::Domain) -> Self {
        Hide(val)
    }
    pub fn from_ref<'s>(rf: &'s F::Domain) -> &'s Self {
        RefCast::ref_cast(rf)
    }
    pub fn reveal(&self) -> &F::Domain {
        &self.0
    }
    pub fn into_reveal(self) -> F::Domain {
        self.0
    }
}

impl<T: Ord + Neg> Neg for Hide<Max<T>>
where
    T::Output: Ord,
{
    type Output = Hide<Min<T::Output>>;
    fn neg(self) -> Self::Output {
        Hide::from_val(self.into_reveal().neg())
    }
}

impl<T: Ord + Neg> Neg for Hide<Min<T>>
where
    T::Output: Ord,
{
    type Output = Hide<Max<T::Output>>;
    fn neg(self) -> Self::Output {
        Hide::from_val(self.into_reveal().neg())
    }
}

impl<K: Eq + Ord, F: Lattice> Hide<MapUnion<BTreeMap<K, F>>> {
    pub fn get(&self, key: &K) -> Hide<RefOptional<'_, F>> {
        let opt = self.reveal().get(key);
        Hide::from_val(opt)
    }
}
impl<K: Eq + Hash, F: Lattice> Hide<MapUnion<HashMap<K, F>>> {
    pub fn get(&self, key: &K) -> Hide<RefOptional<'_, F>> {
        let opt = self.reveal().get(key);
        Hide::from_val(opt)
    }
}

impl<T: Eq + Ord> Hide<Union<BTreeSet<T>>> {
    pub fn iter(&self) -> btree_set::Iter<'_, T> {
        self.reveal().iter()
    }
    pub fn into_iter(self) -> btree_set::IntoIter<T> {
        self.into_reveal().into_iter()
    }
}
impl<T: Eq + Hash> Hide<Union<HashSet<T>>> {
    pub fn iter(&self) -> hash_set::Iter<'_, T> {
        self.reveal().iter()
    }
    pub fn into_iter(self) -> hash_set::IntoIter<T> {
        self.into_reveal().into_iter()
    }
}
