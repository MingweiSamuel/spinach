use std::collections::{ BTreeSet, HashSet, BTreeMap, HashMap };
use std::hash::Hash;
use std::iter::FromIterator;

use crate::types::UnaryFunction;

pub trait Set<T>: SetRead<T> + SetWrite<T> {}
impl <S, T> Set<T> for S where S: SetRead<T> + SetWrite<T> {}

pub trait SetRead<T> {
    fn map<F, C>(&self) -> C
    where
        F: UnaryFunction<Domain = T>,
        C: FromIterator<<F as UnaryFunction>::Codomain> + SetRead<<F as UnaryFunction>::Codomain>;

    fn union<C>(&self, other: &Self) -> C
    where
        C: FromIterator<T>;

    fn intersect<C>(&self, other: &Self) -> C
    where
        C: FromIterator<T>;
}

pub trait SetWrite<T> {
    fn add(&mut self, item: T);

    fn union_in(&mut self, other: Self);

    fn intersect_in(&mut self, other: Self);
}


impl <'a, T> SetRead<&'a T> for &'a BTreeSet<T>
where
    T: Eq + Ord,
{
    fn map<F, C>(&self) -> C
    where
        F: UnaryFunction<Domain = &'a T>,
        C: FromIterator<<F as UnaryFunction>::Codomain> + SetRead<<F as UnaryFunction>::Codomain>
    {
        self.iter()
            .map(|x| F::call(x))
            .collect()
    }

    fn union<C>(&self, other: &Self) -> C
    where
        C: FromIterator<&'a T>,
    {
        (*self).union(*other)
            .collect()
    }

    fn intersect<C>(&self, other: &Self) -> C
    where
        C: FromIterator<&'a T>,
    {
        (*self).intersection(*other)
            .collect()
    }
}

impl <'a, K, V> SetRead<(&'a K, &'a V)> for &'a BTreeMap<K, V>
where
    K: Eq + Ord,
{
    fn map<F, C>(&self) -> C
    where
        F: UnaryFunction<Domain = (&'a K, &'a V)>,
        C: FromIterator<<F as UnaryFunction>::Codomain> + SetRead<<F as UnaryFunction>::Codomain>
    {
        self.iter()
            .map(|x| F::call(x))
            .collect()
    }

    fn union<C>(&self, other: &Self) -> C
    where
        C: FromIterator<(&'a K, &'a V)>,
    {
        let other_new = (*other).iter().filter(|(k, _)| !(*self).contains_key(k));
        (*self).iter()
            .chain(other_new)
            .collect()
    }

    fn intersect<C>(&self, other: &Self) -> C
    where
        C: FromIterator<(&'a K, &'a V)>,
    {
        (*self).iter()
            .filter(|(k, _)| (*other).contains_key(k))
            .collect()
    }
}









// impl <T> SetRead<T> for BTreeSet<T>
// where
//     T: Eq + Ord + Clone,
// {
//     fn map<F, C>(&self) -> C
//     where
//         F: UnaryFunction<Domain = T>,
//         C: FromIterator<<F as UnaryFunction>::Codomain> + SetRead<<F as UnaryFunction>::Codomain>
//     {
//         self.iter()
//             .map(|x| F::call(x))
//             .collect()
//     }

//     fn union<C>(&self, other: &Self) -> C
//     where
//         T: Clone,
//         C: FromIterator<T>,
//     {
//         self.union(other)
//             .cloned()
//             .collect()
//     }

//     fn intersect<C>(&self, other: &Self) -> C
//     where
//         T: Clone,
//         C: FromIterator<T>,
//     {
//         self.intersection(other)
//             .cloned()
//             .collect()
//     }
// }

// impl <T> SetRead<T> for HashSet<T>
// where
//     T: Eq + Hash,
// {
//     fn map<'a, F, C>(&'a self) -> C
//     where
//         T: 'a,
//         F: UnaryFunction<Domain = &'a T>,
//         C: FromIterator<<F as UnaryFunction>::Codomain> + SetRead<<F as UnaryFunction>::Codomain>
//     {
//         self.iter()
//             .map(|x| F::call(x))
//             .collect()
//     }

//     fn union<C>(&self, other: &Self) -> C
//     where
//         T: Clone,
//         C: FromIterator<T>,
//     {
//         self.union(other)
//             .cloned()
//             .collect()
//     }

//     fn intersect<C>(&self, other: &Self) -> C
//     where
//         T: Clone,
//         C: FromIterator<T>,
//     {
//         self.intersection(other)
//             .cloned()
//             .collect()
//     }
// }

// impl <K, V> SetRead<(K, V)> for BTreeMap<K, V>
// where
//     T: Eq + Ord,
// {
//     fn map<'a, F, C>(&'a self) -> C
//     where
//         T: 'a,
//         F: UnaryFunction<Domain = &'a (K, V)>,
//         C: FromIterator<<F as UnaryFunction>::Codomain> + SetRead<<F as UnaryFunction>::Codomain>
//     {
//         self.iter()
//             .map(|x| F::call(x))
//             .collect()
//     }

//     fn union<C>(&self, other: &Self) -> C
//     where
//         (K, V): Clone,
//         C: FromIterator<(K, V)>,
//     {
//         panic!();
//         // self.union(other)
//         //     .cloned()
//         //     .collect()
//     }

//     fn intersect<C>(&self, other: &Self) -> C
//     where
//         (K, V): Clone,
//         C: FromIterator<(K, V)>,
//     {
//         panic!();
//         // self.intersection(other)
//         //     .cloned()
//         //     .collect()
//     }
// }



