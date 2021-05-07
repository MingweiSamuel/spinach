use collections::*;

pub trait Lattice {}

pub trait LatticeRepr {
    type Lattice: Lattice;
    type Repr;
}

pub trait Merge<Delta: LatticeRepr>: LatticeRepr<Lattice = Delta::Lattice> {
    fn merge(this: &mut Self::Repr, delta: Delta::Repr);
}

pub trait Convert<Target: LatticeRepr<Lattice = Self::Lattice>>: LatticeRepr {
    fn convert(this: Self::Repr) -> Target::Repr;
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

    pub struct SetUnionBind<X> {
        _phantom: std::marker::PhantomData<X>,
    }

    impl<T: Eq + Hash> LatticeRepr for SetUnionBind<HashSet<T>> {
        type Lattice = SetUnion<T>;
        type Repr = HashSet<T>;
    }
    impl<T: Eq + Ord> LatticeRepr for SetUnionBind<BTreeSet<T>> {
        type Lattice = SetUnion<T>;
        type Repr = BTreeSet<T>;
    }
    impl<T: Eq> LatticeRepr for SetUnionBind<Vec<T>> {
        type Lattice = SetUnion<T>;
        type Repr = Vec<T>;
    }
    impl<T: Eq> LatticeRepr for SetUnionBind<Single<T>> {
        type Lattice = SetUnion<T>;
        type Repr = Single<T>;
    }
    impl<T: Eq> LatticeRepr for SetUnionBind<Option<T>> {
        type Lattice = SetUnion<T>;
        type Repr = Option<T>;
    }
    impl<T: Eq, const N: usize> LatticeRepr for SetUnionBind<Array<T, N>> {
        type Lattice = SetUnion<T>;
        type Repr = Array<T, N>;
    }
    impl<T: Eq, const N: usize> LatticeRepr for SetUnionBind<MaskedArray<T, N>> {
        type Lattice = SetUnion<T>;
        type Repr = MaskedArray<T, N>;
    }

    // impl<T, This: LatticeRepr<Lattice = SetUnion<T>>, Delta: LatticeRepr<Lattice = SetUnion<T>>> Merge<SetUnion<T>, Delta> for This
    // where
    //     This::Repr: Extend<T>,
    //     Delta::Repr: IntoIterator<Item = T>,
    // {
    //     fn merge(this: &mut Self::Repr, delta: Delta::Repr) {
    //         this.extend(delta)
    //     }
    // }

    impl<T, X, Y> Merge<SetUnionBind<Y>> for SetUnionBind<X>
    where
        SetUnionBind<X>: LatticeRepr<Lattice = SetUnion<T>>,
        SetUnionBind<Y>: LatticeRepr<Lattice = SetUnion<T>>,
        <SetUnionBind<X> as LatticeRepr>::Repr: Extend<T>,
        <SetUnionBind<Y> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    {
        fn merge(this: &mut <SetUnionBind<X> as LatticeRepr>::Repr, delta: <SetUnionBind<Y> as LatticeRepr>::Repr) {
            this.extend(delta)
        }
    }

    impl<T, X, Y> Convert<SetUnionBind<Y>> for SetUnionBind<X>
    where
        SetUnionBind<X>: LatticeRepr<Lattice = SetUnion<T>>,
        SetUnionBind<Y>: LatticeRepr<Lattice = SetUnion<T>>,
        <SetUnionBind<X> as LatticeRepr>::Repr: IntoIterator<Item = T>,
        <SetUnionBind<Y> as LatticeRepr>::Repr: FromIterator<T>,
    {
        fn convert(this: <SetUnionBind<X> as LatticeRepr>::Repr) -> <SetUnionBind<Y> as LatticeRepr>::Repr {
            this.into_iter().collect()
        }
    }

    fn __assert_merges() {
        use static_assertions::{assert_impl_all, assert_not_impl_any};

        assert_impl_all!(SetUnionBind<HashSet<u32>>:
            Merge<SetUnionBind<HashSet<u32>>>,
            Merge<SetUnionBind<BTreeSet<u32>>>,
            Merge<SetUnionBind<Vec<u32>>>,
            Merge<SetUnionBind<Single<u32>>>,
            Merge<SetUnionBind<Option<u32>>>,
            Merge<SetUnionBind<Array<u32, 8>>>,
            Merge<SetUnionBind<MaskedArray<u32, 8>>>,
        );

        assert_impl_all!(SetUnionBind<BTreeSet<u32>>:
            Merge<SetUnionBind<HashSet<u32>>>,
            Merge<SetUnionBind<BTreeSet<u32>>>,
            Merge<SetUnionBind<Vec<u32>>>,
            Merge<SetUnionBind<Single<u32>>>,
            Merge<SetUnionBind<Option<u32>>>,
            Merge<SetUnionBind<Array<u32, 8>>>,
            Merge<SetUnionBind<MaskedArray<u32, 8>>>,
        );

        assert_impl_all!(SetUnionBind<Vec<u32>>:
            Merge<SetUnionBind<HashSet<u32>>>,
            Merge<SetUnionBind<BTreeSet<u32>>>,
            Merge<SetUnionBind<Vec<u32>>>,
            Merge<SetUnionBind<Single<u32>>>,
            Merge<SetUnionBind<Option<u32>>>,
            Merge<SetUnionBind<Array<u32, 8>>>,
            Merge<SetUnionBind<MaskedArray<u32, 8>>>,
        );

        assert_not_impl_any!(Single<Vec<u32>>:
            Merge<SetUnionBind<HashSet<u32>>>,
            Merge<SetUnionBind<BTreeSet<u32>>>,
            Merge<SetUnionBind<Vec<u32>>>,
            Merge<SetUnionBind<Single<u32>>>,
            Merge<SetUnionBind<Option<u32>>>,
            Merge<SetUnionBind<Array<u32, 8>>>,
            Merge<SetUnionBind<MaskedArray<u32, 8>>>,
        );
    }
}

pub mod mapunion {
    use super::*;

    use std::collections::{BTreeMap, HashMap};
    use std::hash::Hash;

    pub struct MapUnion<K, L: Lattice> {
        _phantom: std::marker::PhantomData<(K, L)>,
    }
    impl<K, L: Lattice> Lattice for MapUnion<K, L> {}

    pub struct MapUnionBind<K, B: LatticeRepr> {
        _phantom: std::marker::PhantomData<(K, B)>,
    }

    impl<K: Eq + Hash, B: LatticeRepr> LatticeRepr for MapUnionBind<HashMap<K, ()>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = HashMap<K, B::Repr>;
    }
    impl<K: Eq + Ord, B: LatticeRepr> LatticeRepr for MapUnionBind<BTreeMap<K, ()>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = BTreeMap<K, B::Repr>;
    }
    impl<K: Eq, B: LatticeRepr> LatticeRepr for MapUnionBind<Vec<(K, ())>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = Vec<(K, B::Repr)>;
    }
    impl<K: Eq, B: LatticeRepr> LatticeRepr for MapUnionBind<Single<(K, ())>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = Single<(K, B::Repr)>;
    }
    impl<K: Eq, B: LatticeRepr> LatticeRepr for MapUnionBind<Option<(K, ())>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = Option<(K, B::Repr)>;
    }
    impl<K: Eq, B: LatticeRepr, const N: usize> LatticeRepr for MapUnionBind<Array<(K, ()), N>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = Array<(K, B::Repr), N>;
    }
    impl<K: Eq, B: LatticeRepr, const N: usize> LatticeRepr for MapUnionBind<MaskedArray<(K, ()), N>, B> {
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = MaskedArray<(K, B::Repr), N>;
    }

    impl<K, X, Y, B: LatticeRepr<Lattice = L>, C: LatticeRepr<Lattice = L>, L: Lattice> Merge<MapUnionBind<Y, C>> for MapUnionBind<X, B>
    where
        MapUnionBind<X, B>: LatticeRepr<Lattice = MapUnion<K, L>>,
        MapUnionBind<Y, C>: LatticeRepr<Lattice = MapUnion<K, L>>,
        <MapUnionBind<X, B> as LatticeRepr>::Repr: Extend<(K, B::Repr)> + Dict<K, B::Repr>,
        <MapUnionBind<Y, C> as LatticeRepr>::Repr: IntoIterator<Item = (K, C::Repr)>,
        B: Merge<C>,
        C: Convert<B>,
    {
        fn merge(this: &mut <MapUnionBind<X, B> as LatticeRepr>::Repr, delta: <MapUnionBind<Y, C> as LatticeRepr>::Repr) {
            let iter: Vec<(K, B::Repr)> = delta.into_iter()
                .filter_map(|(k, v)| {
                    match this.get_mut(&k) {
                        Some(target_val) => {
                            <B as Merge<C>>::merge(target_val, v);
                            None
                        }
                        None => {
                            let val: B::Repr = <C as Convert<B>>::convert(v);
                            Some((k, val))
                        }
                    }
                })
                .collect();
            this.extend(iter);
        }
    }

    fn __assert_merges() {
        use std::collections::{HashSet};
        use static_assertions::{assert_impl_all, assert_not_impl_any};
        
        use super::setunion::{SetUnionBind, SetUnion};

        type HashMapHashSet  = MapUnionBind<HashMap<String, ()>, SetUnionBind<HashSet<u32>>>;
        type HashMapArraySet = MapUnionBind<HashMap<String, ()>, SetUnionBind<Array<u32, 8>>>;
        type OptionMapArraySet = MapUnionBind<Option<(String, ())>, SetUnionBind<HashSet<u32>>>;

        // // Option B: Pattern matchable
        // type HashMapHashSet  = MapUnionBind<HashMap<String, HashSet<u32>>, SetUnionBind<HashSet<u32>>>;
        
        // type MySet = HashSet<u32>;
        // type HashMapHashSet  = MapUnionBind<HashMap<String, MySet>, SetUnionBind<MySet>>;

        // // Split the split difference.
        // type HashMapHashSet  = MapUnionBind<HashMap<String, Smth>, SetUnionBind<HashSet<u32>>>;
        // // Split the difference.
        // // Option A: Tag
        // type HashMapHashSet  = MapUnionBind<HashMapTag<String>, SetUnionBind<ThisIsAHashSet<u32>>>;

        assert_impl_all!(HashMapHashSet: Merge<HashMapHashSet>);
        assert_impl_all!(HashMapHashSet: Merge<HashMapArraySet>);

        assert_not_impl_any!(HashMapArraySet: Merge<HashMapHashSet>);
        assert_not_impl_any!(HashMapArraySet: Merge<HashMapArraySet>);

        assert_not_impl_any!(OptionMapArraySet: Merge<HashMapHashSet>);
        assert_not_impl_any!(OptionMapArraySet: Merge<HashMapArraySet>);
    }
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