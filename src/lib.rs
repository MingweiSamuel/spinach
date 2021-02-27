#![allow(clippy::redundant_closure)]
#![allow(incomplete_features)]
#![doc(include = "../README.md")]
#![feature(cell_update)]
#![feature(drain_filter)]
#![feature(generic_associated_types)]
#![feature(try_blocks)]
#![feature(type_alias_impl_trait)]
#![feature(btree_retain)]
#![feature(external_doc)]
#![forbid(unsafe_code)]

pub mod func;

pub mod lattice;

pub mod comp;

pub mod op;

pub mod flow;

pub mod monotonic;
