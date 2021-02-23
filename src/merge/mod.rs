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
