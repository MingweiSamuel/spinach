#![allow(clippy::redundant_closure)]
#![allow(incomplete_features)]

#![doc = include_str!("../../README.md")]

#![feature(array_map)]
#![feature(array_methods)]
#![feature(array_zip)]
#![feature(associated_type_defaults)]
#![feature(cell_update)]
#![feature(drain_filter)]
#![feature(generic_associated_types)]
#![feature(try_blocks)]
#![feature(min_type_alias_impl_trait)]
#![feature(never_type)]

#![forbid(unsafe_code)]

pub mod collections;

pub mod func;

pub mod tag;

pub mod lattice;

pub mod hide;

pub mod op;

pub mod comp;

pub mod metadata;
