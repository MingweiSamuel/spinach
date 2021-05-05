use std::cmp::Ordering;

use collections::*;

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



pub trait Lattice {}

pub trait LatticeBind {
    type Lattice: Lattice;
    type Repr;
}

pub trait Merge<L: Lattice, Delta: LatticeBind<Lattice = L>>: LatticeBind<Lattice = L> {
    fn merge(this: &mut Self::Repr, delta: Delta::Repr);
}

pub trait Convert<L: Lattice, Target: LatticeBind<Lattice = L>>: LatticeBind<Lattice = L> {
    fn convert(this: Self::Repr) -> Target::Repr;
}

pub struct LP<B: LatticeBind> {
    value: B::Repr,
}

// impl<L: Lattice, R: Repr<L>> LP<L, R> {
//     pub fn merge<S: Repr<L>>(&mut self, delta: LP<L, S>)
//     where
//         R: Merge<L, S>,
//     {
//         // REVEAL!
//         self.value.merge(delta.value);
//     }
// }

pub mod setunion {
    use super::*;

    use std::collections::{BTreeSet, HashSet};
    use std::hash::Hash;
    use std::iter::FromIterator;

    pub struct SetUnion<T> {
        _phantom: std::marker::PhantomData<T>,
    }
    impl<T> Lattice for SetUnion<T> {}

    pub struct SetUnionBind<X> {
        _phantom: std::marker::PhantomData<X>,
    }

    impl<T: Eq + Hash> LatticeBind for SetUnionBind<HashSet<T>> {
        type Lattice = SetUnion<T>;
        type Repr = HashSet<T>;
    }
    impl<T: Eq + Ord> LatticeBind for SetUnionBind<BTreeSet<T>> {
        type Lattice = SetUnion<T>;
        type Repr = BTreeSet<T>;
    }
    impl<T: Eq> LatticeBind for SetUnionBind<Vec<T>> {
        type Lattice = SetUnion<T>;
        type Repr = Vec<T>;
    }
    impl<T: Eq> LatticeBind for SetUnionBind<Single<T>> {
        type Lattice = SetUnion<T>;
        type Repr = Single<T>;
    }
    impl<T: Eq> LatticeBind for SetUnionBind<Option<T>> {
        type Lattice = SetUnion<T>;
        type Repr = Option<T>;
    }
    impl<T: Eq, const N: usize> LatticeBind for SetUnionBind<Array<T, N>> {
        type Lattice = SetUnion<T>;
        type Repr = Array<T, N>;
    }
    impl<T: Eq, const N: usize> LatticeBind for SetUnionBind<MaskedArray<T, N>> {
        type Lattice = SetUnion<T>;
        type Repr = MaskedArray<T, N>;
    }

    // impl<T, This: LatticeBind<Lattice = SetUnion<T>>, Delta: LatticeBind<Lattice = SetUnion<T>>> Merge<SetUnion<T>, Delta> for This
    // where
    //     This::Repr: Extend<T>,
    //     Delta::Repr: IntoIterator<Item = T>,
    // {
    //     fn merge(this: &mut Self::Repr, delta: Delta::Repr) {
    //         this.extend(delta)
    //     }
    // }

    impl<T, X, Y> Merge<SetUnion<T>, SetUnionBind<Y>> for SetUnionBind<X>
    where
        Self: LatticeBind<Lattice = SetUnion<T>>,
        SetUnionBind<Y>: LatticeBind<Lattice = SetUnion<T>>,
        Self::Repr: Extend<T>,
        <SetUnionBind<Y> as LatticeBind>::Repr: IntoIterator<Item = T>,
    {
        fn merge(this: &mut Self::Repr, delta: <SetUnionBind<Y> as LatticeBind>::Repr) {
            this.extend(delta)
        }
    }

    impl<T, X, Y> Convert<SetUnion<T>, SetUnionBind<Y>> for SetUnionBind<X>
    where
        Self: LatticeBind<Lattice = SetUnion<T>>,
        SetUnionBind<Y>: LatticeBind<Lattice = SetUnion<T>>,
        Self::Repr: IntoIterator<Item = T>,
        <SetUnionBind<Y> as LatticeBind>::Repr: FromIterator<T>,
    {
        fn convert(this: Self::Repr) -> <SetUnionBind<Y> as LatticeBind>::Repr {
            this.into_iter().collect()
        }
    }

    // impl<T, This: LatticeBind<Lattice = SetUnion<T>>, Target: LatticeBind<Lattice = SetUnion<T>>> Convert<Target> for This
    // where
    //     This::Repr: IntoIterator<Item = T>,
    //     Target::Repr: FromIterator<T>,
    // {
    //     fn convert(this: Self::Repr) -> Target::Repr {
    //         this.into_iter().collect()
    //     }
    // }

    // impl<T, Original: Repr<SetUnion<T>> + IntoIterator<Item = T>, Target: Repr<SetUnion<T>> + FromIterator<T>> Convert<SetUnion<T>, Target> for Original {
    //     fn convert(self) -> Target {
    //         self.into_iter().collect()
    //     }
    // }

    // fn __assert_merges() {
    //     use static_assertions::{assert_impl_all, assert_not_impl_any};

    //     assert_impl_all!(HashSet<u32>:
    //         Merge<SetUnion<u32>, HashSet<u32>>,
    //         Merge<SetUnion<u32>, BTreeSet<u32>>,
    //         Merge<SetUnion<u32>, Vec<u32>>,
    //         Merge<SetUnion<u32>, Single<u32>>,
    //         Merge<SetUnion<u32>, Option<u32>>,
    //         Merge<SetUnion<u32>, Array<u32, 8>>,
    //         Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
    //     );

    //     assert_impl_all!(BTreeSet<u32>:
    //         Merge<SetUnion<u32>, HashSet<u32>>,
    //         Merge<SetUnion<u32>, BTreeSet<u32>>,
    //         Merge<SetUnion<u32>, Vec<u32>>,
    //         Merge<SetUnion<u32>, Single<u32>>,
    //         Merge<SetUnion<u32>, Option<u32>>,
    //         Merge<SetUnion<u32>, Array<u32, 8>>,
    //         Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
    //     );

    //     assert_impl_all!(Vec<u32>:
    //         Merge<SetUnion<u32>, HashSet<u32>>,
    //         Merge<SetUnion<u32>, BTreeSet<u32>>,
    //         Merge<SetUnion<u32>, Vec<u32>>,
    //         Merge<SetUnion<u32>, Single<u32>>,
    //         Merge<SetUnion<u32>, Option<u32>>,
    //         Merge<SetUnion<u32>, Array<u32, 8>>,
    //         Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
    //     );

    //     assert_not_impl_any!(Single<u32>:
    //         Merge<SetUnion<u32>, HashSet<u32>>,
    //         Merge<SetUnion<u32>, BTreeSet<u32>>,
    //         Merge<SetUnion<u32>, Vec<u32>>,
    //         Merge<SetUnion<u32>, Single<u32>>,
    //         Merge<SetUnion<u32>, Option<u32>>,
    //         Merge<SetUnion<u32>, Array<u32, 8>>,
    //         Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
    //     );

    //     assert_not_impl_any!(Option<u32>:
    //         Merge<SetUnion<u32>, HashSet<u32>>,
    //         Merge<SetUnion<u32>, BTreeSet<u32>>,
    //         Merge<SetUnion<u32>, Vec<u32>>,
    //         Merge<SetUnion<u32>, Single<u32>>,
    //         Merge<SetUnion<u32>, Option<u32>>,
    //         Merge<SetUnion<u32>, Array<u32, 8>>,
    //         Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
    //     );

    //     assert_not_impl_any!(Array<u32, 8>:
    //         Merge<SetUnion<u32>, HashSet<u32>>,
    //         Merge<SetUnion<u32>, BTreeSet<u32>>,
    //         Merge<SetUnion<u32>, Vec<u32>>,
    //         Merge<SetUnion<u32>, Single<u32>>,
    //         Merge<SetUnion<u32>, Option<u32>>,
    //         Merge<SetUnion<u32>, Array<u32, 8>>,
    //         Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
    //     );

    //     assert_not_impl_any!(MaskedArray<u32, 8>:
    //         Merge<SetUnion<u32>, HashSet<u32>>,
    //         Merge<SetUnion<u32>, BTreeSet<u32>>,
    //         Merge<SetUnion<u32>, Vec<u32>>,
    //         Merge<SetUnion<u32>, Single<u32>>,
    //         Merge<SetUnion<u32>, Option<u32>>,
    //         Merge<SetUnion<u32>, Array<u32, 8>>,
    //         Merge<SetUnion<u32>, MaskedArray<u32, 8>>,
    //     );
    // }
}

pub mod mapunion {
    use super::*;

    use std::collections::{BTreeMap, HashMap};
    use std::hash::Hash;

    pub struct MapUnion<K, L: Lattice> {
        _phantom: std::marker::PhantomData<(K, L)>,
    }
    impl<K, L: Lattice> Lattice for MapUnion<K, L> {}

    pub struct MapUnionBind<T, B: LatticeBind> {
        _phantom: std::marker::PhantomData<(T, B)>,
    }

    impl<K: Eq + Hash, B: LatticeBind> LatticeBind for MapUnionBind<HashMap<K, B>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = HashMap<K, B::Repr>;
    }
    impl<K: Eq + Ord, B: LatticeBind> LatticeBind for MapUnionBind<BTreeMap<K, B>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = BTreeMap<K, B::Repr>;
    }
    impl<K: Eq, B: LatticeBind> LatticeBind for MapUnionBind<Vec<(K, B)>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = Vec<(K, B::Repr)>;
    }
    impl<K: Eq, B: LatticeBind> LatticeBind for MapUnionBind<Single<(K, B)>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = Single<(K, B::Repr)>;
    }
    impl<K: Eq, B: LatticeBind> LatticeBind for MapUnionBind<Option<(K, B)>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = Option<(K, B::Repr)>;
    }
    impl<K: Eq, B: LatticeBind, const N: usize> LatticeBind for MapUnionBind<Array<(K, B), N>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = Array<(K, B::Repr), N>;
    }
    impl<K: Eq, B: LatticeBind, const N: usize> LatticeBind for MapUnionBind<MaskedArray<(K, B), N>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = MaskedArray<(K, B::Repr), N>;
    }

    impl<K, X, Y, B: LatticeBind<Lattice = L>, C: LatticeBind<Lattice = L>, L: Lattice> Merge<MapUnion<K, L>, MapUnionBind<Y, C>> for MapUnionBind<X, B>
    where
        Self: LatticeBind<Lattice = MapUnion<K, B::Lattice>>,
        MapUnionBind<Y, C>: LatticeBind<Lattice = MapUnion<K, C::Lattice>>,
        Self::Repr: Extend<(K, B::Repr)> + Dict<K, B::Repr>,
        <MapUnionBind<Y, C> as LatticeBind>::Repr: IntoIterator<Item = (K, C::Repr)>,
        B: Merge<L, C>,
        C: Convert<L, B>,
    {
        fn merge(this: &mut Self::Repr, delta: <MapUnionBind<Y, C> as LatticeBind>::Repr) {
            let iter: Vec<(K, B::Repr)> = delta.into_iter()
                .filter_map(|(k, v)| {
                    match this.get_mut(&k) {
                        Some(target_val) => {
                            <B as Merge<L, C>>::merge(target_val, v);
                            None
                        }
                        None => {
                            let val: B::Repr = <C as Convert<L, B>>::convert(v);
                            Some((k, val))
                        }
                    }
                })
                .collect();
            this.extend(iter);
        }
    }

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

    // fn bool_to_option<'a>(value: bool) -> Option<&'a ()> {
    //     if value { Some(&()) } else { None }
    // }

    // fn bool_to_option_mut<'a>(value: bool) -> Option<&'a mut ()> {
    //     if value {
    //         Some(&mut ())
    //     } 
    //     else {
    //         None
    //     }
    // }

    pub trait Dict<K, V> {
        fn get(&self, key: &K) -> Option<&V>;
        fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    }

    // impl<K: Eq + Hash> Dict<K, ()> for HashSet<K> {
    //     fn get(&self, key: &K) -> Option<&()> {
    //         bool_to_option(self.contains(key))
    //     }
    //     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
    //         bool_to_option_mut(self.contains(key))
    //     }
    // }

    // impl<K: Eq + Ord> Dict<K, ()> for BTreeSet<K> {
    //     fn get(&self, key: &K) -> Option<&()> {
    //         bool_to_option(self.contains(key))
    //     }
    //     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
    //         bool_to_option_mut(self.contains(key))
    //     }
    // }

    // impl<K: Eq> Dict<K, ()> for Vec<K> {
    //     fn get(&self, key: &K) -> Option<&()> {
    //         bool_to_option(<[K]>::contains(self, key))
    //     }
    //     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
    //         bool_to_option_mut(self.contains(key))
    //     }
    // }

    // impl<K: Eq, const N: usize> Dict<K, ()> for Array<K, N> {
    //     fn get(&self, key: &K) -> Option<&()> {
    //         bool_to_option(self.0.contains(key))
    //     }
    //     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
    //         bool_to_option_mut(self.0.contains(key))
    //     }
    // }

    // impl<K: Eq, const N: usize> Dict<K, ()> for MaskedArray<K, N> {
    //     fn get(&self, key: &K) -> Option<&()> {
    //         bool_to_option(self.mask.iter()
    //                 .zip(self.vals.iter())
    //                 .any(|(mask, item)| *mask && item == key))
    //     }
    //     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
    //         bool_to_option_mut(self.mask.iter()
    //                 .zip(self.vals.iter())
    //                 .any(|(mask, item)| *mask && item == key))
    //     }
    // }




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