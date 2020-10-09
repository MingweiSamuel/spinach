use std::hash::Hash;
use std::collections::{ BTreeSet, HashMap, HashSet };
use std::collections::hash_map::Entry;
use std::iter::Extend;

pub use crate::Lattice;

/// Merge trait.
pub trait Merge<T> {
    // A "static" method.
    fn merge(val: &mut T, other: T);
}

// ORD MERGES //

pub struct MaxMerge;
impl <T: Ord> Merge<T> for MaxMerge {
    fn merge(val: &mut T, other: T) {
        if *val < other {
            *val = other;
        }
    }
}

pub struct MinMerge;
impl <T: Ord> Merge<T> for MinMerge {
    fn merge(val: &mut T, other: T) {
        if *val > other {
            *val = other;
        }
    }
}

// SET MERGES //

pub struct UnionMerge;
impl <T: Eq + Hash> Merge<HashSet<T>> for UnionMerge {
    fn merge(val: &mut HashSet<T>, other: HashSet<T>) {
        val.extend(other);
    }
}
impl <T: Eq + Ord> Merge<BTreeSet<T>> for UnionMerge {
    fn merge(val: &mut BTreeSet<T>, other: BTreeSet<T>) {
        val.extend(other);
    }
}

pub struct IntersectMerge;
impl <T: Eq + Hash> Merge<HashSet<T>> for IntersectMerge {
    fn merge(val: &mut HashSet<T>, other: HashSet<T>) {
        val.retain(|x| other.contains(x));
    }
}
impl <T: Eq + Ord> Merge<BTreeSet<T>> for IntersectMerge {
    fn merge(val: &mut BTreeSet<T>, other: BTreeSet<T>) {
        // Not so ergonomic nor efficient.
        *val = other.into_iter()
            .filter(|x| val.contains(x))
            .collect();
    }
}

// MAP MERGES //

type LatticeMap<K, V, F> = HashMap<K, Lattice<V, F>>;

pub struct MapUnionMerge;
impl <K: Eq + Hash, V, F: Merge<V>> Merge<LatticeMap<K, V, F>> for MapUnionMerge {
    fn merge(val: &mut LatticeMap<K, V, F>, other: LatticeMap<K, V, F>) {
        for (k, v) in other {
            match val.entry(k) {
                Entry::Occupied(mut kv) => {
                    kv.get_mut().merge_in(v.into_reveal());
                },
                Entry::Vacant(kv) => {
                    kv.insert(v);
                },
            }
        }
    }
}

pub struct MapIntersectionMerge;
impl <K: Eq + Hash, V, F: Merge<V>> Merge<LatticeMap<K, V, F>> for MapIntersectionMerge {
    fn merge(val: &mut LatticeMap<K, V, F>, other: LatticeMap<K, V, F>) {
        for (k, v) in other {
            val.entry(k).and_modify(|v0| v0.merge_in(v.into_reveal()));
        }
    }
}
