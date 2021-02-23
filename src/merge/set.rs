use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::iter::Extend;

use super::Merge;

// SET MERGES //

pub struct Union<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T: Eq + Hash> Merge for Union<HashSet<T>> {
    type Domain = HashSet<T>;

    fn merge_in(val: &mut HashSet<T>, delta: HashSet<T>) {
        val.extend(delta);
    }

    fn partial_cmp(val: &HashSet<T>, delta: &HashSet<T>) -> Option<Ordering> {
        let s = val.union(delta).count();
        if s != val.len() && s != delta.len() {
            None
        } else if s == val.len() {
            if s == delta.len() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            Some(Ordering::Less)
        }
    }

    fn remainder(val: &mut Self::Domain, mut delta: Self::Domain) -> bool {
        delta.retain(|item| !val.contains(item));
        *val = delta;
        val.is_empty()
    }
}
impl<T: Eq + Ord> Merge for Union<BTreeSet<T>> {
    type Domain = BTreeSet<T>;

    fn merge_in(val: &mut BTreeSet<T>, delta: BTreeSet<T>) {
        val.extend(delta);
    }

    fn partial_cmp(val: &BTreeSet<T>, delta: &BTreeSet<T>) -> Option<Ordering> {
        let s = val.union(delta).count();
        if s != val.len() && s != delta.len() {
            None
        } else if s == val.len() {
            if s == delta.len() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            }
        } else {
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
impl<T: Eq + Hash> Merge for Intersect<HashSet<T>> {
    type Domain = HashSet<T>;

    fn merge_in(val: &mut HashSet<T>, delta: HashSet<T>) {
        val.retain(|x| delta.contains(x));
    }

    fn partial_cmp(val: &HashSet<T>, delta: &HashSet<T>) -> Option<Ordering> {
        let s = val.intersection(delta).count();
        if s != val.len() && s != delta.len() {
            None
        } else if s == val.len() {
            if s == delta.len() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            Some(Ordering::Less)
        }
    }
}
impl<T: Eq + Ord> Merge for Intersect<BTreeSet<T>> {
    type Domain = BTreeSet<T>;

    fn merge_in(val: &mut BTreeSet<T>, delta: BTreeSet<T>) {
        // Not so ergonomic nor efficient.
        *val = delta.into_iter().filter(|x| val.contains(x)).collect();
    }

    fn partial_cmp(val: &BTreeSet<T>, delta: &BTreeSet<T>) -> Option<Ordering> {
        let s = val.intersection(delta).count();
        if s != val.len() && s != delta.len() {
            None
        } else if s == val.len() {
            if s == delta.len() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            Some(Ordering::Less)
        }
    }
}
