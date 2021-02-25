#![forbid(unsafe_code)]
#![allow(clippy::redundant_closure)]
#![feature(cell_update)]
#![feature(drain_filter)]
#![feature(try_blocks)]
#![feature(type_alias_impl_trait)]
#![feature(btree_retain)]
#![feature(external_doc)]
#![doc(include = "../README.md")]

pub mod func;

pub mod lattice;

pub mod comp;

pub mod op;

pub mod flow;

pub mod monotonic;
