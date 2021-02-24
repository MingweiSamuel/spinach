#![forbid(unsafe_code)]
#![allow(clippy::redundant_closure)]
#![feature(cell_update)]
#![feature(drain_filter)]
// #![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![feature(btree_retain)]
#![feature(external_doc)]
#![doc(include = "../README.md")]

pub mod func;

pub mod merge;

pub mod op;

pub mod monotonic;
