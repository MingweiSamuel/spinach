use std::cmp::Ordering;

use super::Lattice;

// ORD MERGES //

/// For totally-ordered demains, take the max value.
pub struct Max<T: Ord> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T: Ord + Clone> Lattice for Max<T> {
    type Domain = T;

    fn merge_in(val: &mut T, delta: T) {
        if *val < delta {
            *val = delta;
        }
    }

    fn partial_cmp(val: &T, delta: &T) -> Option<Ordering> {
        val.partial_cmp(delta)
    }

    fn delta(val: &Self::Domain, delta: &mut Self::Domain) -> bool {
        *delta > *val
    }
}

/// For totally-ordered demains, take the min value.
pub struct Min<T: Ord> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T: Ord + Clone> Lattice for Min<T> {
    type Domain = T;

    fn merge_in(val: &mut T, delta: T) {
        if *val > delta {
            *val = delta;
        }
    }

    fn partial_cmp(val: &T, delta: &T) -> Option<Ordering> {
        val.partial_cmp(delta).map(|ord| ord.reverse())
    }

    fn delta(val: &Self::Domain, delta: &mut Self::Domain) -> bool {
        *delta < *val
    }
}
