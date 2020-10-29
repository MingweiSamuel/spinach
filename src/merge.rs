use std::hash::Hash;
use std::collections::{ BTreeMap, BTreeSet, HashMap, HashSet };
use std::collections::hash_map;
use std::collections::btree_map;
use std::iter::Extend;

use std::cmp::Ordering;

/// Merge trait.
pub trait Merge {
    type Domain;

    // A "static" method.
    fn merge(val: &mut Self::Domain, other: Self::Domain);

    fn partial_cmp(val: &Self::Domain, other: &Self::Domain) -> Option<Ordering>;
}



// Mingwei's weird semilattice.
// Merge is defined as, given signed integers A and B, take the value in the
// range [A, B] (or [B, A]) which is closest to zero.
// (Note that in general this will be A, B, or zero).
pub struct RangeToZeroMergeI32;
impl Merge for RangeToZeroMergeI32 {
    type Domain = i32;

    fn merge(val: &mut i32, other: i32) {
        if val.signum() != other.signum() {
            *val = 0;
        }
        else if val.abs() > other.abs() {
            *val = other
        }
    }

    fn partial_cmp(val: &i32, other: &i32) -> Option<Ordering> {
        if val.signum() != other.signum() {
            None
        }
        else {
            let less = val.abs().cmp(&other.abs());
            Some(less.reverse())
        }
    }
}




// ORD MERGES //

pub struct MaxMerge<T: Ord> {
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Ord> Merge for MaxMerge<T> {
    type Domain = T;

    fn merge(val: &mut T, other: T) {
        if *val < other {
            *val = other;
        }
    }

    fn partial_cmp(val: &T, other: &T) -> Option<Ordering> {
        val.partial_cmp(other)
    }
}

pub struct MinMerge<T: Ord> {
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Ord> Merge for MinMerge<T> {
    type Domain = T;

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

pub struct UnionMerge<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Eq + Hash> Merge for UnionMerge<HashSet<T>> {
    type Domain = HashSet<T>;

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
impl <T: Eq + Ord> Merge for UnionMerge<BTreeSet<T>> {
    type Domain = BTreeSet<T>;

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

pub struct IntersectMerge<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Eq + Hash> Merge for IntersectMerge<HashSet<T>> {
    type Domain = HashSet<T>;

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
impl <T: Eq + Ord> Merge for IntersectMerge<BTreeSet<T>> {
    type Domain = BTreeSet<T>;

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

pub struct MapUnionMerge<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl <K, F> Merge for MapUnionMerge<HashMap<K, F>>
where
    K: Hash + Eq,
    F: Merge,
{
    type Domain = HashMap<K, <F as Merge>::Domain>;

    fn merge(val: &mut Self::Domain, other: Self::Domain) {
        for (k, v) in other {
            match val.entry(k) {
                hash_map::Entry::Occupied(mut kv) => {
                    F::merge(kv.get_mut(), v);
                },
                hash_map::Entry::Vacant(kv) => {
                    kv.insert(v);
                },
            }
        }
    }

    // TODO: these are awful looking, and also need testing. Could use helper method.
    fn partial_cmp(val: &Self::Domain, other: &Self::Domain) -> Option<Ordering> {
        // Ordering::Equal OR Ordering::Greater
        if val.len() >= other.len() {
            let mut result = None;
            for (k, other_val) in other {
                match val.get(k) {
                    Some(val_val) => {
                        let cmp = F::partial_cmp(val_val, other_val);
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
                        let cmp = F::partial_cmp(val_val, other_val);
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

impl <K, F> Merge for MapUnionMerge<BTreeMap<K, F>>
where
    K: Ord + Eq,
    F: Merge,
{
    type Domain = BTreeMap<K, <F as Merge>::Domain>;

    fn merge(val: &mut Self::Domain, other: Self::Domain) {
        for (k, v) in other {
            match val.entry(k) {
                btree_map::Entry::Occupied(mut kv) => {
                    F::merge(kv.get_mut(), v);
                },
                btree_map::Entry::Vacant(kv) => {
                    kv.insert(v);
                },
            }
        }
    }

    // TODO: these are awful looking, and also need testing. Could use helper method.
    fn partial_cmp(val: &Self::Domain, other: &Self::Domain) -> Option<Ordering> {
        // Ordering::Equal OR Ordering::Greater
        if val.len() >= other.len() {
            let mut result = None;
            for (k, other_val) in other {
                match val.get(k) {
                    Some(val_val) => {
                        let cmp = F::partial_cmp(val_val, other_val);
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
                        let cmp = F::partial_cmp(val_val, other_val);
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

// pub struct MapIntersectionMerge<V, F>
// where
//     F: Merge<V>,
// {
//     _phantom: std::marker::PhantomData<(V, F)>,
// }
// impl <K: Eq + Hash, V, F: Merge<V>> Merge<HashMap<K, V>> for MapIntersectionMerge<V, F> {
//     fn merge(val: &mut HashMap<K, V>, other: HashMap<K, V>) {
//         for (k, v) in other {
//             val.entry(k).and_modify(|v0| F::merge(v0, v));
//         }
//     }

//     fn partial_cmp(val: &HashMap<K, V>, other: &HashMap<K, V>) -> Option<Ordering> {
//         // Ordering::Equal OR Ordering::Greater
//         if val.len() >= other.len() {
//             let mut result = None;
//             for (k, other_val) in other {
//                 match val.get(k) {
//                     Some(val_val) => {
//                         let cmp = F::partial_cmp(&val_val, other_val);
//                         match cmp {
//                             Some(cmp) => {
//                                 if result.get_or_insert(cmp) != &cmp {
//                                     return None;
//                                 }
//                             },
//                             None => return None,
//                         }
//                     },
//                     None => return None,
//                 }
//             }
//             if None == result {
//                 return Some(Ordering::Equal);
//             }
//             else {
//                 return Some(Ordering::Greater);
//             }
//         }
//         // Ordering::Less
//         else {
//             for (k, val_val) in val {
//                 match other.get(k) {
//                     Some(other_val) => {
//                         let cmp = F::partial_cmp(&val_val, other_val);
//                         if Some(Ordering::Less) != cmp {
//                             return None;
//                         }
//                     },
//                     None => return None,
//                 }
//             }
//             return Some(Ordering::Less);
//         }
//     }
// }

pub struct DominatingPairMerge<AF, BF>
where
    AF: Merge,
    BF: Merge,
{
    _phantom: std::marker::PhantomData<(AF, BF)>,
}

impl <AF, BF> Merge for DominatingPairMerge<AF, BF>
where
    AF: Merge,
    BF: Merge,
{
    type Domain = (<AF as Merge>::Domain, <BF as Merge>::Domain);

    fn merge(val: &mut Self::Domain, other: Self::Domain) {
        let cmp = AF::partial_cmp(&val.0, &other.0);
        match cmp {
            None => {
                AF::merge(&mut val.0, other.0);
                BF::merge(&mut val.1, other.1);
            },
            Some(Ordering::Equal) => {
                BF::merge(&mut val.1, other.1);
            },
            Some(Ordering::Less) => {
                *val = other;
            },
            Some(Ordering::Greater) => {},
        }
    }

    fn partial_cmp(val: &Self::Domain, other: &Self::Domain) -> Option<Ordering> {
        AF::partial_cmp(&val.0, &other.0).or_else(|| BF::partial_cmp(&val.1, &other.1))
    }
}
