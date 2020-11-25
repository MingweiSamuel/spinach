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
// SharedRefPipe <--- SharedMovePipe
//      ^                    ^
//      |                    |
//  ExclRefPipe  <---  ExclMovePipe
//

pub trait Pipe {
    type Item;
}

pub trait SharedRefPipe: Pipe {
    type Feedback: Future;

    #[must_use]
    fn push(&self, item: &<Self as Pipe>::Item) -> Self::Feedback;
}

pub trait ExclRefPipe: Pipe {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: &<Self as Pipe>::Item) -> Self::Feedback;
}

pub trait SharedMovePipe: Pipe {
    type Feedback: Future;

    #[must_use]
    fn push(&self, item: <Self as Pipe>::Item) -> Self::Feedback;
}

pub trait ExclMovePipe: Pipe {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: <Self as Pipe>::Item) -> Self::Feedback;
}
