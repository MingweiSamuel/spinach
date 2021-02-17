mod casts;
pub use casts::*;

mod impls;
pub use impls::*;

use std::future::Future;



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
    type Indomain;
}

// pub trait Deltaflow<F: crate::merge::Merge>: Op<Item = F::Indomain> {}

pub trait SharedRefOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&self, item: &Self::Indomain) -> Self::Feedback;
}

pub trait ExclRefOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: &Self::Indomain) -> Self::Feedback;
}

pub trait SharedMoveOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&self, item: Self::Indomain) -> Self::Feedback;
}

pub trait ExclMoveOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: Self::Indomain) -> Self::Feedback;
}
