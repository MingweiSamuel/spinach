#![allow(clippy::redundant_closure)]
#![allow(incomplete_features)]

#![doc(include = "../README.md")]

#![feature(array_map)]
#![feature(array_methods)]
#![feature(array_zip)]
#![feature(cell_update)]
#![feature(drain_filter)]
#![feature(generic_associated_types)]
#![feature(try_blocks)]
#![feature(min_type_alias_impl_trait)]
#![feature(external_doc)]

#![forbid(unsafe_code)]

pub mod func;

pub mod lattice;

pub mod lattice2;

// pub mod comp;

pub mod op;

pub mod monotonic;
