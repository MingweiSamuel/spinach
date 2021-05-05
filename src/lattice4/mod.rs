use std::cmp::Ordering;

use collections::*;



pub trait Lattice {}

pub trait Repr<L: Lattice> {}

pub trait LatticeAndRepr {
    type Lattice: Lattice;
    type Repr: Repr<Self::Lattice>;
}

pub trait Merge<L: Lattice, R: Repr<L>>: Repr<L> {
    fn merge(&mut self, delta: R);
}

pub trait Convert<L: Lattice, R: Repr<L>>: Repr<L> {
    fn convert(self) -> R;
}


pub struct LP<L: Lattice, R: Repr<L>> {
    value: R,
    _phantom: std::marker::PhantomData<L>,
}

impl<L: Lattice, R: Repr<L>> LP<L, R> {
    pub fn merge<S: Repr<L>>(&mut self, delta: LP<L, S>)
    where
        R: Merge<L, S>,
    {
        // REVEAL!
        self.value.merge(delta.value);
    }
}

pub mod setunion {
    use super::*;

    use std::collections::{BTreeSet, HashSet};
    use std::hash::Hash;
    use std::iter::FromIterator;

    pub struct SetUnion<T> {
        _phantom: std::marker::PhantomData<T>,
    }
    impl<T> Lattice for SetUnion<T> {}

    pub struct SetUnionBoth<T, R: Repr<SetUnion<T>>> {
        _phantom: std::marker::PhantomData<(T, R)>,
    }
    impl<T, R: Repr<SetUnion<T>>> LatticeAndRepr for SetUnionBoth<T, R> {
        type Lattice = SetUnion<T>;
        type Repr = R;
    }

    impl<T: Eq + Hash> Repr<SetUnion<T>> for HashSet<T> {}
    impl<T: Eq + Ord> Repr<SetUnion<T>> for BTreeSet<T> {}
    impl<T> Repr<SetUnion<T>> for Vec<T> {}
    impl<T> Repr<SetUnion<T>> for Single<T> {}
    impl<T> Repr<SetUnion<T>> for Option<T> {}
    impl<T, const N: usize> Repr<SetUnion<T>> for Array<T, N> {}
    impl<T, const N: usize> Repr<SetUnion<T>> for MaskedArray<T, N> {}
    

    impl<T, Target: Repr<SetUnion<T>> + Extend<T>, Delta: Repr<SetUnion<T>> + IntoIterator<Item = T>> Merge<SetUnion<T>, Delta> for Target {
        fn merge(&mut self, delta: Delta) {
            self.extend(delta);
        }
    }

    impl<T, Original: Repr<SetUnion<T>>  + IntoIterator<Item = T>, Target: Repr<SetUnion<T>> + FromIterator<T>> Convert<SetUnion<T>, Target> for Original {
        fn convert(self) -> Target {
            self.into_iter().collect()
        }
    }

    fn __assert_merges() {
        use static_assertions::{assert_impl_all, assert_not_impl_any};

        assert_impl_all!(HashSet<u32>:
            Merge<SetUnion<u32>, HashSet<u32>>,
            Merge<SetUnion<u32>, BTreeSet<u32>>,
            Merge<SetUnion<u32>, Vec<u32>>,
            Merge<SetUnion<u32>, Single<u32>>,
            Merge<SetUnion<u32>, Option<u32>>,
            Merge<SetUnion<u32>, Array<u32, 8>>,
            Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
        );

        assert_impl_all!(BTreeSet<u32>:
            Merge<SetUnion<u32>, HashSet<u32>>,
            Merge<SetUnion<u32>, BTreeSet<u32>>,
            Merge<SetUnion<u32>, Vec<u32>>,
            Merge<SetUnion<u32>, Single<u32>>,
            Merge<SetUnion<u32>, Option<u32>>,
            Merge<SetUnion<u32>, Array<u32, 8>>,
            Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
        );

        assert_impl_all!(Vec<u32>:
            Merge<SetUnion<u32>, HashSet<u32>>,
            Merge<SetUnion<u32>, BTreeSet<u32>>,
            Merge<SetUnion<u32>, Vec<u32>>,
            Merge<SetUnion<u32>, Single<u32>>,
            Merge<SetUnion<u32>, Option<u32>>,
            Merge<SetUnion<u32>, Array<u32, 8>>,
            Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
        );

        assert_not_impl_any!(Single<u32>:
            Merge<SetUnion<u32>, HashSet<u32>>,
            Merge<SetUnion<u32>, BTreeSet<u32>>,
            Merge<SetUnion<u32>, Vec<u32>>,
            Merge<SetUnion<u32>, Single<u32>>,
            Merge<SetUnion<u32>, Option<u32>>,
            Merge<SetUnion<u32>, Array<u32, 8>>,
            Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
        );

        assert_not_impl_any!(Option<u32>:
            Merge<SetUnion<u32>, HashSet<u32>>,
            Merge<SetUnion<u32>, BTreeSet<u32>>,
            Merge<SetUnion<u32>, Vec<u32>>,
            Merge<SetUnion<u32>, Single<u32>>,
            Merge<SetUnion<u32>, Option<u32>>,
            Merge<SetUnion<u32>, Array<u32, 8>>,
            Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
        );

        assert_not_impl_any!(Array<u32, 8>:
            Merge<SetUnion<u32>, HashSet<u32>>,
            Merge<SetUnion<u32>, BTreeSet<u32>>,
            Merge<SetUnion<u32>, Vec<u32>>,
            Merge<SetUnion<u32>, Single<u32>>,
            Merge<SetUnion<u32>, Option<u32>>,
            Merge<SetUnion<u32>, Array<u32, 8>>,
            Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
        );

        assert_not_impl_any!(MaskedArray<u32, 8>:
            Merge<SetUnion<u32>, HashSet<u32>>,
            Merge<SetUnion<u32>, BTreeSet<u32>>,
            Merge<SetUnion<u32>, Vec<u32>>,
            Merge<SetUnion<u32>, Single<u32>>,
            Merge<SetUnion<u32>, Option<u32>>,
            Merge<SetUnion<u32>, Array<u32, 8>>,
            Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
        );
    }
}

pub mod mapunion {
    use super::*;

    use std::collections::{BTreeMap, HashMap};

    pub struct MapUnion<K, L: Lattice> {
        _phantom: std::marker::PhantomData<(K, L)>,
    }

    pub struct MapUnionBoth<K, R: Repr<MapUnion<K, B::Lattice>>, B: LatticeAndRepr> {
        _phantom: std::marker::PhantomData<(K, R, B)>,
    }
    impl<K, R: Repr<MapUnion<K, B::Lattice>>, B: LatticeAndRepr> LatticeAndRepr for MapUnionBoth<K, R, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = R;
    }

    impl<K, L: Lattice> Lattice for MapUnion<K, L> {}

    impl<K, L: Lattice, R: Repr<L>> Repr<MapUnion<K, L>> for HashMap<K, R> {}
    impl<K, L: Lattice, R: Repr<L>> Repr<MapUnion<K, L>> for BTreeMap<K, R> {}
    impl<K, L: Lattice, R: Repr<L>> Repr<MapUnion<K, L>> for Vec<(K, R)> {}
    impl<K, L: Lattice, R: Repr<L>> Repr<MapUnion<K, L>> for Single<(K, R)> {}
    impl<K, L: Lattice, R: Repr<L>> Repr<MapUnion<K, L>> for Option<(K, R)> {}
    impl<K, L: Lattice, R: Repr<L>, const N: usize> Repr<MapUnion<K, L>> for Array<(K, R), N> {}
    impl<K, L: Lattice, R: Repr<L>, const N: usize> Repr<MapUnion<K, L>> for MaskedArray<(K, R), N> {}

    // type MyVectorClockVersionedHashMap =
    //     MapUnion<Key,
    //         DominatingPair<
    //             MapUnion<Id, Max<usize>>,
    //             Max<String>
    //         >
    //     >
    // ;

    // type MyVCRepr = HashMap<Key, (HashMap<Id, usize>, String)>

    // type MyBoth =
    //     MapUnion<HashMap<
    //         Key,
    //         DominatingPair<
    //             MapUnion<HashMap<Id, Max<usize, usize>>,
    //             SetUnion<Vec<String>>,
    //         >
    //     >>;
    


    // impl<K, L: Lattice, Target, Delta, TargetRepr, DeltaRepr> Merge<MapUnion<K, L>, Delta> for Target
    // where
    //     Target: Repr<MapUnion<K, L>> + Extend<(K, TargetRepr)> + Dict<K, TargetRepr>,
    //     Delta: Repr<MapUnion<K, L>> + IntoIterator<Item = (K, DeltaRepr)>,
    //     TargetRepr: Repr<L> + Merge<L, DeltaRepr>,
    //     DeltaRepr: Repr<L> + Convert<L, TargetRepr>,
    // {
    //     fn merge(&mut self, delta: Delta) {
    //         let iter = delta.into_iter()
    //             .filter_map(|(k, v)| {
    //                 match self.get_mut(&k) {
    //                     Some(target_val) => {
    //                         target_val.merge(v);
    //                         None
    //                     }
    //                     None => Some((k, v.convert()))
    //                 }
    //             });
    //         self.extend(iter);
    //     }
    // }

    // fn __assert_merges() {
    //     use static_assertions::{assert_impl_all, assert_not_impl_any};

    //     assert_impl_all!(HashSet<u32>:
    //         Merge<MapUnion<String, HashSet<u32>>, HashSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, BTreeSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Vec<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Single<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Option<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Array<u32, 8>>,
    //         Merge<MapUnion<String, HashSet<u32>>, MaskedArray<u32, 8>>,
    //     );

    //     assert_impl_all!(BTreeSet<u32>:
    //         Merge<MapUnion<String, HashSet<u32>>, HashSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, BTreeSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Vec<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Single<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Option<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Array<u32, 8>>,
    //         Merge<MapUnion<String, HashSet<u32>>, MaskedArray<u32, 8>>,
    //     );

    //     assert_impl_all!(Vec<u32>:
    //         Merge<MapUnion<String, HashSet<u32>>, HashSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, BTreeSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Vec<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Single<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Option<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Array<u32, 8>>,
    //         Merge<MapUnion<String, HashSet<u32>>, MaskedArray<u32, 8>>,
    //     );

    //     assert_not_impl_any!(Single<u32>:
    //         Merge<MapUnion<String, HashSet<u32>>, HashSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, BTreeSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Vec<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Single<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Option<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Array<u32, 8>>,
    //         Merge<MapUnion<String, HashSet<u32>>, MaskedArray<u32, 8>>,
    //     );

    //     assert_not_impl_any!(Option<u32>:
    //         Merge<MapUnion<String, HashSet<u32>>, HashSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, BTreeSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Vec<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Single<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Option<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Array<u32, 8>>,
    //         Merge<MapUnion<String, HashSet<u32>>, MaskedArray<u32, 8>>,
    //     );

    //     assert_not_impl_any!(Array<u32, 8>:
    //         Merge<MapUnion<String, HashSet<u32>>, HashSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, BTreeSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Vec<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Single<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Option<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Array<u32, 8>>,
    //         Merge<MapUnion<String, HashSet<u32>>, MaskedArray<u32, 8>>,
    //     );

    //     assert_not_impl_any!(MaskedArray<u32, 8>:
    //         Merge<MapUnion<String, HashSet<u32>>, HashSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, BTreeSet<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Vec<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Single<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Option<u32>>,
    //         Merge<MapUnion<String, HashSet<u32>>, Array<u32, 8>>,
    //         Merge<MapUnion<String, HashSet<u32>>, MaskedArray<u32, 8>>,
    //     );
    // }
}







pub mod collections {
    use std::array::IntoIter;
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
    use std::hash::Hash;

    fn bool_to_option<'a>(value: bool) -> Option<&'a ()> {
        if value { Some(&()) } else { None }
    }

    fn bool_to_option_mut<'a>(value: bool) -> Option<&'a mut ()> {
        if value { Some(&mut ()) } else { None }
    }

    pub trait Dict<K, V> {
        fn get(&self, key: &K) -> Option<&V>;
        fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    }

    impl<K: Eq + Hash> Dict<K, ()> for HashSet<K> {
        fn get(&self, key: &K) -> Option<&()> {
            bool_to_option(self.contains(key))
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
            bool_to_option_mut(self.contains(key))
        }
    }

    impl<K: Eq + Ord> Dict<K, ()> for BTreeSet<K> {
        fn get(&self, key: &K) -> Option<&()> {
            bool_to_option(self.contains(key))
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
            bool_to_option_mut(self.contains(key))
        }
    }

    impl<K: Eq> Dict<K, ()> for Vec<K> {
        fn get(&self, key: &K) -> Option<&()> {
            bool_to_option(<[K]>::contains(self, key))
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
            bool_to_option_mut(self.contains(key))
        }
    }

    impl<K: Eq, const N: usize> Dict<K, ()> for Array<K, N> {
        fn get(&self, key: &K) -> Option<&()> {
            bool_to_option(self.0.contains(key))
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
            bool_to_option_mut(self.0.contains(key))
        }
    }

    impl<K: Eq, const N: usize> Dict<K, ()> for MaskedArray<K, N> {
        fn get(&self, key: &K) -> Option<&()> {
            bool_to_option(self.mask.iter()
                    .zip(self.vals.iter())
                    .any(|(mask, item)| *mask && item == key))
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
            bool_to_option_mut(self.mask.iter()
                    .zip(self.vals.iter())
                    .any(|(mask, item)| *mask && item == key))
        }
    }




    impl<K: Eq + Hash, V> Dict<K, V> for HashMap<K, V> {
        fn get(&self, key: &K) -> Option<&V> {
            self.get(key)
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut V> {
            self.get_mut(key)
        }
    }

    impl<K: Eq + Ord, V> Dict<K, V> for BTreeMap<K, V> {
        fn get(&self, key: &K) -> Option<&V> {
            self.get(key)
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut V> {
            self.get_mut(key)
        }
    }

    impl<K: Eq, V> Dict<K, V> for Vec<(K, V)> {
        fn get(&self, key: &K) -> Option<&V> {
            self.iter()
                .find(|(k, _)| k == key)
                .map(|(_, val)| val)
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut V> {
            self.iter_mut()
                .find(|(k, _)| k == key)
                .map(|(_, val)| val)
        }
    }

    impl<K: Eq, V, const N: usize> Dict<K, V> for Array<(K, V), N> {
        fn get(&self, key: &K) -> Option<&V> {
            self.0.iter()
                .find(|(k, _)| k == key)
                .map(|(_, val)| val)
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut V> {
            self.0.iter_mut()
                .find(|(k, _)| k == key)
                .map(|(_, val)| val)
        }
    }

    impl<K: Eq, V, const N: usize> Dict<K, V> for MaskedArray<(K, V), N> {
        fn get(&self, key: &K) -> Option<&V> {
            self.mask.iter()
                .zip(self.vals.iter())
                .find(|(mask, (k, _))| **mask && k == key)
                .map(|(_, (_, val))| val)
        }
        fn get_mut(&mut self, key: &K) -> Option<&mut V> {
            self.mask.iter()
                .zip(self.vals.iter_mut())
                .find(|(mask, (k, _))| **mask && k == key)
                .map(|(_, (_, val))| val)
        }
    }


    #[repr(transparent)]
    pub struct Single<T>(pub T);
    impl<T> IntoIterator for Single<T> {
        type Item = T;
        type IntoIter = <Option<T> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            Some(self.0).into_iter()
        }
    }


    pub struct Array<T, const N: usize>(pub [T; N]);
    impl<T, const N: usize> IntoIterator for Array<T, N> {
        type Item = T;
        type IntoIter = IntoIter<T, N>;

        fn into_iter(self) -> Self::IntoIter {
            IntoIter::new(self.0)
        }
    }


    pub struct MaskedArray<T, const N: usize> {
        pub mask: [bool; N],
        pub vals: [T; N],
    }
    impl<T, const N: usize> IntoIterator for MaskedArray<T, N> {
        type Item = T;
        type IntoIter = impl Iterator<Item = Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            IntoIter::new(self.mask)
                .zip(IntoIter::new(self.vals))
                .filter(|(mask, _)| *mask)
                .map(|(_, val)| val)
        }
    }
}