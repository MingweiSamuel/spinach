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
    fn insert(&mut self, item: T);

    fn union_in(&mut self, other: Self);

    fn intersect_in(&mut self, other: Self);
}

macro_rules! setwrite_set_impl {
    ( $type:ident, $bound:ident ) => {
        impl <T> SetWrite<T> for $type<T>
        where
            T: Eq + $bound,
        {
            fn insert(&mut self, item: T) {
                self.insert(item);
            }

            fn union_in(&mut self, other: Self) {
                self.extend(other.into_iter());
            }

            fn intersect_in(&mut self, other: Self) {
                // Not clean or efficient.
                *self = other.into_iter()
                    .filter(|x| self.contains(x))
                    .collect();
            }
        }
    };
}
setwrite_set_impl!(BTreeSet, Ord);
setwrite_set_impl!(HashSet, Hash);


macro_rules! setwrite_map_impl {
    ( $type:ident, $bound:ident ) => {
        impl <K, V> SetWrite<(K, V)> for $type<K, V>
        where
            K: Eq + $bound,
            V: Eq,
        {
            fn insert(&mut self, item: (K, V)) {
                self.insert(item.0, item.1);
            }

            fn union_in(&mut self, other: Self) {
                self.extend(other.into_iter());
            }

            fn intersect_in(&mut self, other: Self) {
                for (k, other_val) in other.into_iter() {
                    if self.get(&k) != Some(&other_val) {
                        self.remove(&k);
                    }
                }
            }
        }
    };
}
setwrite_map_impl!(BTreeMap, Ord);
setwrite_map_impl!(HashMap, Hash);




macro_rules! setread_set_impl {
    ( $type:ident, $bound:ident ) => {
        impl <'a, T> SetRead<&'a T> for &'a $type<T>
        where
            T: Eq + $bound,
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
    };
}
setread_set_impl!(BTreeSet, Ord);
setread_set_impl!(HashSet, Hash);

macro_rules! setread_map_impl {
    ( $type:ident, $bound:ident ) => {
        impl <'a, K, V> SetRead<(&'a K, &'a V)> for &'a $type<K, V>
        where
            K: Eq + $bound,
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
    };
}
setread_map_impl!(BTreeMap, Ord);
setread_map_impl!(HashMap, Hash);

