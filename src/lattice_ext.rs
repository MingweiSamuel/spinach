use std::collections::{ BTreeSet, HashSet, HashMap };
use std::hash::Hash;

use crate::merge::{ Merge, UnionMerge, MapUnionMerge };
use crate::lattice::Semilattice;
use crate::types::UnaryFunction;

impl <T> Semilattice<UnionMerge<BTreeSet<T>>>
where
    T: Eq + Ord,
{
    pub fn map<F>(self) -> Semilattice<UnionMerge<BTreeSet<<F as UnaryFunction>::Codomain>>>
    where
        F: UnaryFunction<Domain = T>,
        <F as UnaryFunction>::Codomain: Eq + Ord,
    {
        let val = self.into_reveal() // Reveal here!
            .into_iter()
            .map(|x| F::call(x))
            .collect();

        Semilattice::new(val)
    }
}

impl <T> Semilattice<UnionMerge<HashSet<T>>>
where
    T: Eq + Hash,
{
    pub fn map<F>(self) -> Semilattice<UnionMerge<HashSet<<F as UnaryFunction>::Codomain>>>
    where
        F: UnaryFunction<Domain = T>,
        <F as UnaryFunction>::Codomain: Eq + Hash,
    {
        let val = self.into_reveal() // Reveal here!
            .into_iter()
            .map(|x| F::call(x))
            .collect();

        Semilattice::new(val)
    }
}

impl <K, M> Semilattice<MapUnionMerge<HashMap<K, M>>>
where
    K: Eq + Hash,
    M: Merge,
{
    pub fn map<F>(self) -> Semilattice<UnionMerge<HashSet<<F as UnaryFunction>::Codomain>>>
    where
        F: UnaryFunction<Domain = ( K, <M as Merge>::Domain )>,
        <F as UnaryFunction>::Codomain: Eq + Hash,
    {
        let val = self.into_reveal() // Reveal here!
            .into_iter()
            .map(|x| F::call(x))
            .collect();

        Semilattice::new(val)
    }
}
