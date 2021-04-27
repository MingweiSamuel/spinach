//! All lattice merges.

use std::cmp::Ordering;

/// Lattice trait, a merge function which defines a lattice.
pub trait Lattice {
    type Domain: Clone;

    fn merge_in(val: &mut Self::Domain, delta: Self::Domain);

    fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering>;

    /// Given an current VAL, modify delta so it is of a minimum size.
    /// Return TRUE if delta is non-trivial. Return FALSE if delta can be ignored (VAL dominates DELTA).
    fn delta(val: &Self::Domain, delta: &mut Self::Domain) -> bool;

    // /// Given VAL and DELTA, sets VAL to a (preferably minimal) value X such
    // /// that merging X and DELTA gives (the original) VAL.
    // fn remainder(val: &mut Self::Domain, delta: Self::Domain) -> bool {
    //     match Self::partial_cmp(val, &delta) {
    //         Some(Ordering::Less) | Some(Ordering::Equal) => {
    //             // If DELTA dominates VAL then doing nothing satisfies the condition.
    //             // Technically we should set the value to bottom.
    //             true
    //         }
    //         _ => {
    //             // Trivially, X = the merge of VAL and DELTA satisfies the condition.
    //             Self::merge_in(val, delta);
    //             false
    //         }
    //     }
    // }
}

mod ord;
pub use ord::*;

mod set;
pub use set::*;

mod map;
pub use map::*;

mod dominatingpair;
pub use dominatingpair::*;

mod other;
pub use other::*;

mod hide;
pub use hide::*;
