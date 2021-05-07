#![allow(non_camel_case_types)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::collections::{Single, Array, MaskedArray};

pub trait Tag1 {
    type Type<T>;
}

pub trait Tag2 {
    type Type<T, U>;
}


pub enum HASH_SET {}
impl Tag1 for HASH_SET {
    type Type<T> = HashSet<T>;
}

pub enum HASH_MAP {}
impl Tag2 for HASH_MAP {
    type Type<T, U> = HashMap<T, U>;
}


pub enum BTREE_SET {}
impl Tag1 for BTREE_SET {
    type Type<T> = BTreeSet<T>;
}

pub enum BTREE_MAP {}
impl Tag2 for BTREE_MAP {
    type Type<T, U> = BTreeMap<T, U>;
}


pub enum VEC {}
impl Tag1 for VEC {
    type Type<T> = Vec<T>;
}
impl Tag2 for VEC {
    type Type<T, U> = Vec<(T, U)>;
}


pub enum SINGLE {}
impl Tag1 for SINGLE {
    type Type<T> = Single<T>;
}
impl Tag2 for SINGLE {
    type Type<T, U> = Single<(T, U)>;
}


pub enum OPTION {}
impl Tag1 for OPTION {
    type Type<T> = Option<T>;
}
impl Tag2 for OPTION {
    type Type<T, U> = Option<(T, U)>;
}


pub struct ARRAY<const N: usize>([(); N]);
impl<const N: usize> Tag1 for ARRAY<N> {
    type Type<T> = Array<T, N>;
}
impl<const N: usize> Tag2 for ARRAY<N> {
    type Type<T, U> = Array<(T, U), N>;
}


pub struct MASKED_ARRAY<const N: usize>([(); N]);
impl<const N: usize> Tag1 for MASKED_ARRAY<N> {
    type Type<T> = MaskedArray<T, N>;
}
impl<const N: usize> Tag2 for MASKED_ARRAY<N> {
    type Type<T, U> = MaskedArray<(T, U), N>;
}
