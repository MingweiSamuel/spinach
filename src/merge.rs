// use std::hash::Hash;
// use std::collections::{ BTreeSet, HashSet };
// use std::collections::hash_map::Entry;
// use std::iter::Extend;

use crate::kudzu::{ set::Set, map::Map };

// use crate::Lattice;
// use crate::LatticeMap;

pub trait Merge<T> {
    // "static" methods, since they don't have "self".

    /// Merges consumes OTHER and merges it into VAL.
    fn merge(val: &T, other: &T);
}


use std::sync::atomic::{ Ordering, AtomicBool, AtomicUsize };
// TODO: I have no idea which atomics Ordering to use.

pub struct MaxMerge;
impl Merge<AtomicBool> for MaxMerge {
    fn merge(left: &AtomicBool, rght: &AtomicBool) {
        let _old = left.fetch_or(rght.load(Ordering::AcqRel), Ordering::AcqRel);
    }
}
impl Merge<AtomicUsize> for MaxMerge {
    fn merge(left: &AtomicUsize, rght: &AtomicUsize) {
        let _old = left.fetch_max(rght.load(Ordering::AcqRel), Ordering::AcqRel);
    }
}

pub struct UnionMerge;
impl <T: Ord + Copy> Merge<Set<T>> for UnionMerge {
    fn merge(left: &Set<T>, rght: &Set<T>) {
        left.extend(rght.iter().copied());
    }
}

pub struct Lattice<T, F: Merge<T>> {
    val: T,
    phantom: std::marker::PhantomData<F>,
}
