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
    fn merge(val: &T, other: T);
}


use std::sync::atomic::{ Ordering, AtomicBool, AtomicUsize };
// TODO: I have no idea which atomics Ordering to use.

pub struct MaxMerge;
impl Merge<AtomicBool> for MaxMerge {
    fn merge(left: &AtomicBool, rght: AtomicBool) {
        left.fetch_or(rght.load(Ordering::AcqRel), Ordering::AcqRel);
    }
}

pub struct Lattice<T, F: Merge<T>> {
    val: T,
    phantom: std::marker::PhantomData<F>,
}

impl <T, F: Merge<T>> Lattice<T, F> {
    pub fn new(val: T) -> Self {
        Lattice {
            val: val,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn merge(&self, val: T) {
        F::merge(&self.val, val);
    }
}

#[test]
fn test_stuff() {
    let mono_pred: Lattice<AtomicBool, MaxMerge> = Lattice::new(false.into());
    let _mp = mono_pred;
}
