use std::hash::Hash;
use std::collections::{ BTreeSet, HashMap, HashSet };
use std::collections::hash_map::Entry;
use std::iter::Extend;

use std::cmp::Ordering;

use crate::Lattice;

/// Merge trait.
pub trait Merge<T> {
    // A "static" method.
    fn merge(val: &mut T, other: T);

    fn partial_cmp(val: &T, other: &T) -> Option<Ordering>;
}

// ORD MERGES //

pub struct MaxMerge;
impl <T: Ord> Merge<T> for MaxMerge {
    fn merge(val: &mut T, other: T) {
        if *val < other {
            *val = other;
        }
    }

    fn partial_cmp(val: &T, other: &T) -> Option<Ordering> {
        val.partial_cmp(other)
    }
}

pub struct MinMerge;
impl <T: Ord> Merge<T> for MinMerge {
    fn merge(val: &mut T, other: T) {
        if *val > other {
            *val = other;
        }
    }

    fn partial_cmp(val: &T, other: &T) -> Option<Ordering> {
        val.partial_cmp(other).map(|ord| ord.reverse())
    }
}

// SET MERGES //

pub struct UnionMerge;
impl <T: Eq + Hash> Merge<HashSet<T>> for UnionMerge {
    fn merge(val: &mut HashSet<T>, other: HashSet<T>) {
        val.extend(other);
    }

    fn partial_cmp(val: &HashSet<T>, other: &HashSet<T>) -> Option<Ordering> {
        let s = val.union(other).count();
        if s != val.len() && s != other.len() {
            None
        }
        else if s == val.len() {
            if s == other.len() {
                Some(Ordering::Equal)
            }
            else {
                Some(Ordering::Greater)
            }
        }
        else {
            Some(Ordering::Less)
        }
    }
}
impl <T: Eq + Ord> Merge<BTreeSet<T>> for UnionMerge {
    fn merge(val: &mut BTreeSet<T>, other: BTreeSet<T>) {
        val.extend(other);
    }

    fn partial_cmp(val: &BTreeSet<T>, other: &BTreeSet<T>) -> Option<Ordering> {
        let s = val.union(other).count();
        if s != val.len() && s != other.len() {
            None
        }
        else if s == val.len() {
            if s == other.len() {
                Some(Ordering::Equal)
            }
            else {
                Some(Ordering::Greater)
            }
        }
        else {
            Some(Ordering::Less)
        }
    }
}

pub struct IntersectMerge;
impl <T: Eq + Hash> Merge<HashSet<T>> for IntersectMerge {
    fn merge(val: &mut HashSet<T>, other: HashSet<T>) {
        val.retain(|x| other.contains(x));
    }

    fn partial_cmp(val: &HashSet<T>, other: &HashSet<T>) -> Option<Ordering> {
        let s = val.intersection(other).count();
        if s != val.len() && s != other.len() {
            None
        }
        else if s == val.len() {
            if s == other.len() {
                Some(Ordering::Equal)
            }
            else {
                Some(Ordering::Greater)
            }
        }
        else {
            Some(Ordering::Less)
        }
    }
}
impl <T: Eq + Ord> Merge<BTreeSet<T>> for IntersectMerge {
    fn merge(val: &mut BTreeSet<T>, other: BTreeSet<T>) {
        // Not so ergonomic nor efficient.
        *val = other.into_iter()
            .filter(|x| val.contains(x))
            .collect();
    }

    fn partial_cmp(val: &BTreeSet<T>, other: &BTreeSet<T>) -> Option<Ordering> {
        let s = val.intersection(other).count();
        if s != val.len() && s != other.len() {
            None
        }
        else if s == val.len() {
            if s == other.len() {
                Some(Ordering::Equal)
            }
            else {
                Some(Ordering::Greater)
            }
        }
        else {
            Some(Ordering::Less)
        }
    }
}

// MAP MERGES //

pub struct MapUnionMerge;
impl <K: Eq + Hash, V, F: Merge<V>> Merge<HashMap<K, Lattice<V, F>>> for MapUnionMerge {
    fn merge(val: &mut HashMap<K, Lattice<V, F>>, other: HashMap<K, Lattice<V, F>>) {
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

    // TODO: these are awful looking, and also need testing. Could use helper method.
    fn partial_cmp(val: &HashMap<K, Lattice<V, F>>, other: &HashMap<K, Lattice<V, F>>) -> Option<Ordering> {
        // Ordering::Equal OR Ordering::Greater
        if val.len() >= other.len() {
            let mut result = None;
            for (k, other_val) in other {
                match val.get(k) {
                    Some(val_val) => {
                        let cmp = val_val.reveal_partial_cmp(other_val);
                        match cmp {
                            Some(cmp) => {
                                if result.get_or_insert(cmp) != &cmp {
                                    return None;
                                }
                            },
                            None => return None,
                        }
                    },
                    None => return None,
                }
            }
            if None == result {
                return Some(Ordering::Equal);
            }
            else {
                return Some(Ordering::Greater);
            }
        }
        // Ordering::Less
        else {
            for (k, val_val) in val {
                match other.get(k) {
                    Some(other_val) => {
                        let cmp = val_val.reveal_partial_cmp(other_val);
                        if Some(Ordering::Less) != cmp {
                            return None;
                        }
                    },
                    None => return None,
                }
            }
            return Some(Ordering::Less);
        }
    }
}

pub struct MapIntersectionMerge;
impl <K: Eq + Hash, V, F: Merge<V>> Merge<HashMap<K, Lattice<V, F>>> for MapIntersectionMerge {
    fn merge(val: &mut HashMap<K, Lattice<V, F>>, other: HashMap<K, Lattice<V, F>>) {
        for (k, v) in other {
            val.entry(k).and_modify(|v0| v0.merge_in(v.into_reveal()));
        }
    }

    fn partial_cmp(val: &HashMap<K, Lattice<V, F>>, other: &HashMap<K, Lattice<V, F>>) -> Option<Ordering> {
        // Ordering::Equal OR Ordering::Greater
        if val.len() >= other.len() {
            let mut result = None;
            for (k, other_val) in other {
                match val.get(k) {
                    Some(val_val) => {
                        let cmp = val_val.reveal_partial_cmp(other_val);
                        match cmp {
                            Some(cmp) => {
                                if result.get_or_insert(cmp) != &cmp {
                                    return None;
                                }
                            },
                            None => return None,
                        }
                    },
                    None => return None,
                }
            }
            if None == result {
                return Some(Ordering::Equal);
            }
            else {
                return Some(Ordering::Greater);
            }
        }
        // Ordering::Less
        else {
            for (k, val_val) in val {
                match other.get(k) {
                    Some(other_val) => {
                        let cmp = val_val.reveal_partial_cmp(other_val);
                        if Some(Ordering::Less) != cmp {
                            return None;
                        }
                    },
                    None => return None,
                }
            }
            return Some(Ordering::Less);
        }
    }
}

pub struct DominatingPairMerge;
impl <A, B, AF: Merge<A>, BF: Merge<B>> Merge<(Lattice<A, AF>, Lattice<B, BF>)> for DominatingPairMerge {
    fn merge(val: &mut (Lattice<A, AF>, Lattice<B, BF>), other: (Lattice<A, AF>, Lattice<B, BF>)) {
        let cmp = val.0.reveal_partial_cmp(&other.0);
        match cmp {
            None => {
                val.0.merge_in(other.0.into_reveal());
                val.1.merge_in(other.1.into_reveal());
            }
            Some(Ordering::Equal) => {
                val.1.merge_in(other.1.into_reveal());
            }
            Some(Ordering::Greater) => {},
            Some(Ordering::Less) => {
                *val = other;
            },
        };
    }

    fn partial_cmp(val: &(Lattice<A, AF>, Lattice<B, BF>), other: &(Lattice<A, AF>, Lattice<B, BF>)) -> Option<Ordering> {
        val.0.reveal_partial_cmp(&other.0).or_else(|| val.1.reveal_partial_cmp(&other.1))
    }
}