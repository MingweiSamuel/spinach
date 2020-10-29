use std::collections::{ BTreeSet, HashSet, BTreeMap, HashMap };
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

impl <K, F> Semilattice<MapUnionMerge<HashMap<K, F>>>
where
    K: Eq + Hash,
    F: Merge,
{
    pub fn into_kv(self, key: &K) -> Semilattice<F> {
        let ( _, val ) = self.into_reveal()
            .into_iter()
            .find(|( k, v )| key == k)
            .unwrap(); // BAD!!!!!!

        Semilattice::new(val)
    }
}

// impl <K, F> Semilattice<MapUnionMerge<HashMap<K, F>>>
// where
//     K: Eq + Hash,
//     F: Merge,
// {
//     pub fn into_kv(self, key: &K) -> Semilattice<F> {
//         let ( _, val ) = self.into_reveal()
//             .into_iter()
//             .find(|( k, v )| key == k)
//             .unwrap(); // BAD!!!!!!

//         Semilattice::new(val)
//     }
// }
