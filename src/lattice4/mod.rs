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

pub mod tag {
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

    use super::collections::{Single, Array, MaskedArray};

    pub trait Tag1 {
        type Type<T>;
    }

    pub trait Tag2 {
        type Type<T, U>;
    }

    #[allow(non_camel_case_types)]
    pub enum HASH_SET {}
    impl Tag1 for HASH_SET {
        type Type<T> = HashSet<T>;
    }
    #[allow(non_camel_case_types)]
    pub enum HASH_MAP {}
    impl Tag2 for HASH_MAP {
        type Type<T, U> = HashMap<T, U>;
    }

    #[allow(non_camel_case_types)]
    pub enum BTREE_SET {}
    impl Tag1 for BTREE_SET {
        type Type<T> = BTreeSet<T>;
    }
    #[allow(non_camel_case_types)]
    pub enum BTREE_MAP {}
    impl Tag2 for BTREE_MAP {
        type Type<T, U> = BTreeMap<T, U>;
    }

    #[allow(non_camel_case_types)]
    pub enum VEC {}
    impl Tag1 for VEC {
        type Type<T> = Vec<T>;
    }
    impl Tag2 for VEC {
        type Type<T, U> = Vec<(T, U)>;
    }

    #[allow(non_camel_case_types)]
    pub enum SINGLE {}
    impl Tag1 for SINGLE {
        type Type<T> = Single<T>;
    }
    impl Tag2 for SINGLE {
        type Type<T, U> = Single<(T, U)>;
    }

    #[allow(non_camel_case_types)]
    pub enum OPTION {}
    impl Tag1 for OPTION {
        type Type<T> = Option<T>;
    }
    impl Tag2 for OPTION {
        type Type<T, U> = Option<(T, U)>;
    }

    #[allow(non_camel_case_types)]
    pub struct ARRAY<const N: usize>([(); N]);
    impl<const N: usize> Tag1 for ARRAY<N> {
        type Type<T> = Array<T, N>;
    }
    impl<const N: usize> Tag2 for ARRAY<N> {
        type Type<T, U> = Array<(T, U), N>;
    }

    #[allow(non_camel_case_types)]
    pub struct MASKED_ARRAY<const N: usize>([(); N]);
    impl<const N: usize> Tag1 for MASKED_ARRAY<N> {
        type Type<T> = MaskedArray<T, N>;
    }
    impl<const N: usize> Tag2 for MASKED_ARRAY<N> {
        type Type<T, U> = MaskedArray<(T, U), N>;
    }
}

pub mod setunion {
    use super::*;

    use std::iter::FromIterator;

    pub struct SetUnion<T> {
        _phantom: std::marker::PhantomData<T>,
    }
    impl<T> Lattice for SetUnion<T> {}

    pub trait SetTag: tag::Tag1 {}
    impl SetTag for tag::HASH_SET {}
    impl SetTag for tag::BTREE_SET {}
    impl SetTag for tag::VEC {}
    impl SetTag for tag::SINGLE {}
    impl SetTag for tag::OPTION {}
    impl<const N: usize> SetTag for tag::ARRAY<N> {}
    impl<const N: usize> SetTag for tag::MASKED_ARRAY<N> {}

    pub struct SetUnionRepr<Tag: SetTag, T> {
        _phantom: std::marker::PhantomData<(Tag, T)>,
    }

    impl<Tag: SetTag, T> LatticeRepr for SetUnionRepr<Tag, T> {
        type Lattice = SetUnion<T>;
        type Repr = Tag::Type<T>;
    }

    impl<T, SelfTag: SetTag, DeltaTag: SetTag> Merge<SetUnionRepr<DeltaTag, T>> for SetUnionRepr<SelfTag, T>
    where
        SetUnionRepr<SelfTag,  T>: LatticeRepr<Lattice = SetUnion<T>>,
        SetUnionRepr<DeltaTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
        <SetUnionRepr<SelfTag,  T> as LatticeRepr>::Repr: Extend<T>,
        <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    {
        fn merge(this: &mut <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr, delta: <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr) {
            this.extend(delta)
        }
    }

    impl<T, SelfTag: SetTag, TargetTag: SetTag> Convert<SetUnionRepr<TargetTag, T>> for SetUnionRepr<SelfTag, T>
    where
        SetUnionRepr<SelfTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
        SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
        <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
        <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: FromIterator<T>,
    {
        fn convert(this: <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr) -> <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr {
            this.into_iter().collect()
        }
    }

    fn __assert_merges() {
        use static_assertions::{assert_impl_all, assert_not_impl_any};

        assert_impl_all!(SetUnionRepr<tag::HASH_SET, u32>:
            Merge<SetUnionRepr<tag::HASH_SET, u32>>,
            Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
            Merge<SetUnionRepr<tag::VEC, u32>>,
            Merge<SetUnionRepr<tag::SINGLE, u32>>,
            Merge<SetUnionRepr<tag::OPTION, u32>>,
            Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
            Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
        );

        assert_impl_all!(SetUnionRepr<tag::BTREE_SET, u32>:
            Merge<SetUnionRepr<tag::HASH_SET, u32>>,
            Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
            Merge<SetUnionRepr<tag::VEC, u32>>,
            Merge<SetUnionRepr<tag::SINGLE, u32>>,
            Merge<SetUnionRepr<tag::OPTION, u32>>,
            Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
            Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
        );

        assert_impl_all!(SetUnionRepr<tag::VEC, u32>:
            Merge<SetUnionRepr<tag::HASH_SET, u32>>,
            Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
            Merge<SetUnionRepr<tag::VEC, u32>>,
            Merge<SetUnionRepr<tag::SINGLE, u32>>,
            Merge<SetUnionRepr<tag::OPTION, u32>>,
            Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
            Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
        );

        assert_not_impl_any!(SetUnionRepr<tag::MASKED_ARRAY<8>, u32>:
            Merge<SetUnionRepr<tag::HASH_SET, u32>>,
            Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
            Merge<SetUnionRepr<tag::VEC, u32>>,
            Merge<SetUnionRepr<tag::SINGLE, u32>>,
            Merge<SetUnionRepr<tag::OPTION, u32>>,
            Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
            Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
        );
    }
}

pub mod mapunion {
    use super::*;

    pub struct MapUnion<K, L: Lattice> {
        _phantom: std::marker::PhantomData<(K, L)>,
    }
    impl<K, L: Lattice> Lattice for MapUnion<K, L> {}

    pub trait MapTag: tag::Tag2 {}
    impl MapTag for tag::HASH_MAP {}
    impl MapTag for tag::BTREE_MAP {}
    impl MapTag for tag::VEC {}
    impl MapTag for tag::SINGLE {}
    impl MapTag for tag::OPTION {}
    impl<const N: usize> MapTag for tag::ARRAY<N> {}
    impl<const N: usize> MapTag for tag::MASKED_ARRAY<N> {}

    pub struct MapUnionBind<Tag: MapTag, K, B: LatticeRepr> {
        _phantom: std::marker::PhantomData<(Tag, K, B)>,
    }

    impl<Tag: MapTag, K, B: LatticeRepr> LatticeRepr for MapUnionBind<Tag, K, B>{
        type Lattice = MapUnion<K, B::Lattice>;
        type Repr = Tag::Type<K, B::Repr>;
    }

    impl<K, SelfTag, DeltaTag, SelfLR: LatticeRepr<Lattice = L>, DeltaLR: LatticeRepr<Lattice = L>, L: Lattice> Merge<MapUnionBind<DeltaTag, K, DeltaLR>> for MapUnionBind<SelfTag, K, SelfLR>
    where
        SelfTag:  MapTag,
        DeltaTag: MapTag,
        MapUnionBind<SelfTag,  K, SelfLR>: LatticeRepr<Lattice = MapUnion<K, L>>,
        MapUnionBind<DeltaTag, K, DeltaLR>: LatticeRepr<Lattice = MapUnion<K, L>>,
        <MapUnionBind<SelfTag,  K, SelfLR> as LatticeRepr>::Repr: Extend<(K, SelfLR::Repr)> + Dict<K, SelfLR::Repr>,
        <MapUnionBind<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr: IntoIterator<Item = (K, DeltaLR::Repr)>,
        SelfLR:  Merge<DeltaLR>,
        DeltaLR: Convert<SelfLR>,
    {
        fn merge(this: &mut <MapUnionBind<SelfTag, K, SelfLR> as LatticeRepr>::Repr, delta: <MapUnionBind<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr) {
            let iter: Vec<(K, SelfLR::Repr)> = delta.into_iter()
                .filter_map(|(k, v)| {
                    match this.get_mut(&k) {
                        Some(target_val) => {
                            <SelfLR as Merge<DeltaLR>>::merge(target_val, v);
                            None
                        }
                        None => {
                            let val: SelfLR::Repr = <DeltaLR as Convert<SelfLR>>::convert(v);
                            Some((k, val))
                        }
                    }
                })
                .collect();
            this.extend(iter);
        }
    }

    fn __assert_merges() {
        use static_assertions::{assert_impl_all, assert_not_impl_any};
        
        use super::setunion::{SetUnionRepr};

        type HashMapHashSet    = MapUnionBind<tag::HASH_MAP, String, SetUnionRepr<tag::HASH_SET, u32>>;
        type HashMapArraySet   = MapUnionBind<tag::HASH_MAP, String, SetUnionRepr<tag::ARRAY<8>, u32>>;
        type OptionMapArraySet = MapUnionBind<tag::OPTION,   String, SetUnionRepr<tag::HASH_SET, u32>>;

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
    use std::collections::{BTreeMap, HashMap};
    //use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
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