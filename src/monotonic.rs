//! Monotonic functions.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use crate::lattice::{Lattice, MapUnion};

/// Specific type of monotonic function for filtering referenced values.
pub trait MonotonicFilterRefFn {
    type Inmerge: Lattice;
    type Outmerge: Lattice;

    fn call<'a>(
        &self,
        item: &'a <Self::Inmerge as Lattice>::Domain,
    ) -> Option<&'a <Self::Outmerge as Lattice>::Domain>;
}

pub struct MapProject<K, T> {
    key: K,
    _phantom: std::marker::PhantomData<T>,
}
impl<K, T> MapProject<K, T> {
    pub fn new(key: K) -> Self {
        Self {
            key,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<K: Hash + Eq, F: Lattice> MonotonicFilterRefFn for MapProject<K, HashMap<K, F>> {
    type Inmerge = MapUnion<HashMap<K, F>>;
    type Outmerge = F;

    fn call<'a>(&self, item: &'a HashMap<K, F::Domain>) -> Option<&'a F::Domain> {
        item.get(&self.key)
    }
}
impl<K: Ord + Eq, F: Lattice> MonotonicFilterRefFn for MapProject<K, BTreeMap<K, F>> {
    type Inmerge = MapUnion<BTreeMap<K, F>>;
    type Outmerge = F;

    fn call<'a>(&self, item: &'a BTreeMap<K, F::Domain>) -> Option<&'a F::Domain> {
        item.get(&self.key)
    }
}
