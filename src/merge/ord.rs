use std::cmp::Ordering;

use super::Merge;

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
