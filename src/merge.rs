use std::hash::Hash;
use std::collections::{ BTreeMap, BTreeSet, HashMap, HashSet };
use std::collections::hash_map;
use std::collections::btree_map;
use std::iter::Extend;

use std::cmp::Ordering;

/// Merge trait.
pub trait Merge {
    type Domain;

    fn merge_in(val: &mut Self::Domain, delta: Self::Domain);

    fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering>;

    /// Given VAL and DELTA, sets VAL to a (preferably minimal) value X such
    /// that merging X and DELTA gives (the original) VAL.
    fn remainder(val: &mut Self::Domain, delta: Self::Domain) -> bool {
        match Self::partial_cmp(val, &delta) {
            Some(Ordering::Less) | Some(Ordering::Equal) => {
                // If DELTA dominates VAL then doing nothing satisfies the condition.
                // Technically we should set the value to bottom.
                true
            }
            _ => {
                // Trivially, X = the merge of VAL and DELTA satisfies the condition.
                Self::merge_in(val, delta);
                false
            }
        }
    }
}



// ORD MERGES //

pub struct Max<T: Ord> {
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Ord> Merge for Max<T> {
    type Domain = T;

    fn merge_in(val: &mut T, delta: T) {
        if *val < delta {
            *val = delta;
        }
    }

    fn partial_cmp(val: &T, delta: &T) -> Option<Ordering> {
        val.partial_cmp(delta)
    }
}

pub struct Min<T: Ord> {
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Ord> Merge for Min<T> {
    type Domain = T;

    fn merge_in(val: &mut T, delta: T) {
        if *val > delta {
            *val = delta;
        }
    }

    fn partial_cmp(val: &T, delta: &T) -> Option<Ordering> {
        val.partial_cmp(delta).map(|ord| ord.reverse())
    }
}

// SET MERGES //

pub struct Union<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Eq + Hash> Merge for Union<HashSet<T>> {
    type Domain = HashSet<T>;

    fn merge_in(val: &mut HashSet<T>, delta: HashSet<T>) {
        val.extend(delta);
    }

    fn partial_cmp(val: &HashSet<T>, delta: &HashSet<T>) -> Option<Ordering> {
        let s = val.union(delta).count();
        if s != val.len() && s != delta.len() {
            None
        }
        else if s == val.len() {
            if s == delta.len() {
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

    fn remainder(val: &mut Self::Domain, mut delta: Self::Domain) -> bool {
        delta.retain(|item| !val.contains(item));
        *val = delta;
        val.is_empty()
    }
}
impl <T: Eq + Ord> Merge for Union<BTreeSet<T>> {
    type Domain = BTreeSet<T>;

    fn merge_in(val: &mut BTreeSet<T>, delta: BTreeSet<T>) {
        val.extend(delta);
    }

    fn partial_cmp(val: &BTreeSet<T>, delta: &BTreeSet<T>) -> Option<Ordering> {
        let s = val.union(delta).count();
        if s != val.len() && s != delta.len() {
            None
        }
        else if s == val.len() {
            if s == delta.len() {
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

    fn remainder(val: &mut Self::Domain, mut delta: Self::Domain) -> bool {
        delta.retain(|item| !val.contains(item));
        *val = delta;
        val.is_empty()
    }
}

pub struct Intersect<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Eq + Hash> Merge for Intersect<HashSet<T>> {
    type Domain = HashSet<T>;

    fn merge_in(val: &mut HashSet<T>, delta: HashSet<T>) {
        val.retain(|x| delta.contains(x));
    }

    fn partial_cmp(val: &HashSet<T>, delta: &HashSet<T>) -> Option<Ordering> {
        let s = val.intersection(delta).count();
        if s != val.len() && s != delta.len() {
            None
        }
        else if s == val.len() {
            if s == delta.len() {
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
impl <T: Eq + Ord> Merge for Intersect<BTreeSet<T>> {
    type Domain = BTreeSet<T>;

    fn merge_in(val: &mut BTreeSet<T>, delta: BTreeSet<T>) {
        // Not so ergonomic nor efficient.
        *val = delta.into_iter()
            .filter(|x| val.contains(x))
            .collect();
    }

    fn partial_cmp(val: &BTreeSet<T>, delta: &BTreeSet<T>) -> Option<Ordering> {
        let s = val.intersection(delta).count();
        if s != val.len() && s != delta.len() {
            None
        }
        else if s == val.len() {
            if s == delta.len() {
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

pub struct MapUnion<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl <K, F> Merge for MapUnion<HashMap<K, F>>
where
    K: Hash + Eq,
    F: Merge,
{
    type Domain = HashMap<K, <F as Merge>::Domain>;

    fn merge_in(val: &mut Self::Domain, delta: Self::Domain) {
        for ( k, v ) in delta {
            match val.entry(k) {
                hash_map::Entry::Occupied(mut kv) => {
                    F::merge_in(kv.get_mut(), v);
                },
                hash_map::Entry::Vacant(kv) => {
                    kv.insert(v);
                },
            }
        }
    }

    // TODO: these are awful looking, and also need testing. Could use helper method.
    fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering> {
        // Ordering::Equal OR Ordering::Greater
        if val.len() >= delta.len() {
            let mut result = None;
            for ( k, delta_val ) in delta {
                match val.get(k) {
                    Some(val_val) => {
                        let cmp = F::partial_cmp(val_val, delta_val);
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
            for ( k, val_val ) in val {
                match delta.get(k) {
                    Some(delta_val) => {
                        let cmp = F::partial_cmp(val_val, delta_val);
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

    fn remainder(val: &mut Self::Domain, delta: Self::Domain) -> bool {
        for ( k, v ) in delta {
            match val.entry(k) {
                hash_map::Entry::Occupied(mut kv) => {
                    if F::remainder(kv.get_mut(), v) { // If value is dominated, remove it.
                        kv.remove_entry();
                    }
                }
                hash_map::Entry::Vacant(kv) => {
                    kv.insert(v);
                }
            }
        }
        val.is_empty()
    }
}

impl <K, F> Merge for MapUnion<BTreeMap<K, F>>
where
    K: Ord + Eq,
    F: Merge,
{
    type Domain = BTreeMap<K, <F as Merge>::Domain>;

    fn merge_in(val: &mut Self::Domain, delta: Self::Domain) {
        for (k, v) in delta {
            match val.entry(k) {
                btree_map::Entry::Occupied(mut kv) => {
                    F::merge_in(kv.get_mut(), v);
                },
                btree_map::Entry::Vacant(kv) => {
                    kv.insert(v);
                },
            }
        }
    }

    // TODO: these are awful looking, and also need testing. Could use helper method.
    fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering> {
        // Ordering::Equal OR Ordering::Greater
        if val.len() >= delta.len() {
            let mut result = None;
            for (k, delta_val) in delta {
                match val.get(k) {
                    Some(val_val) => {
                        let cmp = F::partial_cmp(val_val, delta_val);
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
                match delta.get(k) {
                    Some(delta_val) => {
                        let cmp = F::partial_cmp(val_val, delta_val);
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

    fn remainder(val: &mut Self::Domain, delta: Self::Domain) -> bool {
        for ( k, v ) in delta {
            match val.entry(k) {
                btree_map::Entry::Occupied(mut kv) => {
                    if F::remainder(kv.get_mut(), v) { // If value is dominated, remove it.
                        kv.remove_entry();
                    }
                }
                btree_map::Entry::Vacant(kv) => {
                    kv.insert(v);
                }
            }
        }
        val.is_empty()
    }
}

// pub struct MapIntersection<T> {
//     _phantom: std::marker::PhantomData<T>,
// }
// impl <K, F> Merge for MapIntersection<HashMap<K, F>>
// where
//     K: Eq + Hash,
//     F: Merge,
// {
//     type Domain = HashMap<K, <F as Merge>::Domain>;

//     fn merge_in(val: &mut Self::Domain, delta: Self::Domain) {
//         todo!("this is broken.");
//         for (k, v) in delta {
//             val.entry(k).and_modify(|v0| F::merge_in(v0, v));
//         }
//     }

//     fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering> {
//         todo!("this is broken.");
//         // Ordering::Equal OR Ordering::Less
//         if val.len() >= delta.len() {
//             let mut result = None;
//             for (k, delta_val) in delta {
//                 match val.get(k) {
//                     Some(val_val) => {
//                         let cmp = F::partial_cmp(&val_val, delta_val);
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
//                 return Some(Ordering::Less);
//             }
//         }
//         // Ordering::Greater
//         else {
//             for (k, val_val) in val {
//                 match delta.get(k) {
//                     Some(delta_val) => {
//                         let cmp = F::partial_cmp(&val_val, delta_val);
//                         if Some(Ordering::Greater) != cmp {
//                             return None;
//                         }
//                     },
//                     None => return None,
//                 }
//             }
//             return Some(Ordering::Greater);
//         }
//     }
// }

pub struct DominatingPair<AF, BF>
where
    AF: Merge,
    BF: Merge,
{
    _phantom: std::marker::PhantomData<(AF, BF)>,
}

impl <AF, BF> Merge for DominatingPair<AF, BF>
where
    AF: Merge,
    BF: Merge,
{
    type Domain = (<AF as Merge>::Domain, <BF as Merge>::Domain);

    fn merge_in(val: &mut Self::Domain, delta: Self::Domain) {
        match AF::partial_cmp(&val.0, &delta.0) {
            None => {
                AF::merge_in(&mut val.0, delta.0);
                BF::merge_in(&mut val.1, delta.1);
            },
            Some(Ordering::Equal) => {
                BF::merge_in(&mut val.1, delta.1);
            },
            Some(Ordering::Less) => {
                *val = delta;
            },
            Some(Ordering::Greater) => {},
        }
    }

    fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering> {
        AF::partial_cmp(&val.0, &delta.0).or_else(|| BF::partial_cmp(&val.1, &delta.1))
    }

    fn remainder(val: &mut Self::Domain, delta: Self::Domain) -> bool {
        match AF::partial_cmp(&val.0, &delta.0) {
            None => {
                AF::merge_in(&mut val.0, delta.0);
                BF::remainder(&mut val.1, delta.1);
                false
            }
            Some(Ordering::Equal) => {
                BF::remainder(&mut val.1, delta.1);
                false
            }
            Some(Ordering::Less) => {
                *val = delta;
                false
            }
            Some(Ordering::Greater) => {
                true
            }
        }
    }
}







// Mingwei's weird semilattice.
// Merge is defined as, given signed integers A and B, take the value in the
// range [A, B] (or [B, A]) which is closest to zero.
// (Note that in general this will be A, B, or zero).
pub struct RangeToZeroI32;
impl Merge for RangeToZeroI32 {
    type Domain = i32;

    fn merge_in(val: &mut i32, delta: i32) {
        if val.signum() != delta.signum() {
            *val = 0;
        }
        else if val.abs() > delta.abs() {
            *val = delta
        }
    }

    fn partial_cmp(val: &i32, delta: &i32) -> Option<Ordering> {
        if val.signum() != delta.signum() {
            None
        }
        else {
            let less = val.abs().cmp(&delta.abs());
            Some(less.reverse())
        }
    }
}
