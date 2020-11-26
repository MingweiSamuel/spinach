mod casts;
pub use casts::*;

mod impls;
pub use impls::*;


use std::future::Future;
// use std::fmt::Debug;
// // use std::sync::mpsc;

// use futures;
// use futures::future::{ Either, FutureExt };

// use tokio::sync::mpsc;
// // use tokio::sync::broadcast;

// // use tokio::stream::Stream;

// use crate::merge::Merge;
// // use crate::semilattice::Semilattice;


pub trait UnaryFn<I> {
    type Output;

    fn call(&self, input: I) -> Self::Output;
}


//
// SharedRefOp <--- SharedMoveOp
//      ^                 ^
//      |                 |
//  ExclRefOp  <---  ExclMoveOp
//

pub trait Op {
    type Item;
}

pub trait SharedRefOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&self, item: &Self::Item) -> Self::Feedback;
}

pub trait ExclRefOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: &Self::Item) -> Self::Feedback;
}

pub trait SharedMoveOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&self, item: Self::Item) -> Self::Feedback;
}

pub trait ExclMoveOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: Self::Item) -> Self::Feedback;
}
