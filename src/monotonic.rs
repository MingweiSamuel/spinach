use std::collections::{ HashMap, BTreeMap };
use std::hash::Hash;

use crate::merge::{ Merge, MapUnion };

pub trait MonotonicFilterRefFn {
    type Inmerge: Merge;
    type Outmerge: Merge;

    fn call<'a>(&self, item: &'a <Self::Inmerge as Merge>::Domain) -> Option<&'a <Self::Outmerge as Merge>::Domain>;
}

pub struct MapProject<K, T> {
    key: K,
    _phantom: std::marker::PhantomData<T>,
}
impl<K, T> MapProject<K, T> {
    pub fn new(key: K) -> Self {
        Self {
            key: key,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<K: Hash + Eq, F: Merge> MonotonicFilterRefFn for MapProject<K, HashMap<K, F>> {
    type Inmerge = MapUnion<HashMap<K, F>>;
    type Outmerge = F;

    fn call<'a>(&self, item: &'a HashMap<K, F::Domain>) -> Option<&'a F::Domain> {
        item.get(&self.key)
    }
}
impl<K: Ord + Eq, F: Merge> MonotonicFilterRefFn for MapProject<K, BTreeMap<K, F>> {
    type Inmerge = MapUnion<BTreeMap<K, F>>;
    type Outmerge = F;

    fn call<'a>(&self, item: &'a BTreeMap<K, F::Domain>) -> Option<&'a F::Domain> {
        item.get(&self.key)
    }
}
