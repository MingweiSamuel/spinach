use std::collections::{ BTreeSet, HashSet };

use crate::merge::Merge;
use crate::lattice::Semilattice;
use crate::types::UnaryFunction;

impl <T, F> Semilattice<F>
where
    F: Merge<Domain = HashSet<T>>,
{
    pub fn the_map_function<U>()
    where
        U: UnaryFunction<Domain = <F as Merge>::Domain, Codomain = <F as Merge>::Domain>,
    {
        // do something.
        // Semilattice<
    }
}
