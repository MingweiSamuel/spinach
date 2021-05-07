use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::collections::{Single, Array, MaskedArray};

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
