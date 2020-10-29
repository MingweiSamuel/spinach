use derivative::Derivative;

use crate::merge::Merge;
use crate::lattice::Semilattice;


#[derive(Derivative)]
#[derivative(PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Tag<A, F>
where
    F: Merge,
{
    pub tag: A,

    #[derivative(PartialOrd="ignore")]
    #[derivative(Ord="ignore")]
    #[derivative(PartialEq="ignore")]
    #[derivative(Hash="ignore")]
    pub val: Semilattice<F>,
}

impl <A, F> Tag<A, F>
where
    F: Merge,
{
    // Kinda redundant.
    pub fn new(tag: A, val: Semilattice<F>) -> Self {
        Self {
            tag: tag,
            val: val,
        }
    }
}
