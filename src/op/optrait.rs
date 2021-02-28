use std::future::Future;
use std::task::{Context, Poll};

use crate::flow::*;

/// An empty trait indicating a struct can be used as an Op.
pub trait Op {}

/// A pull-based op, specifying an Outflow domain/flow type.
pub trait PullOp: Op {
    type Outflow: Flow;
    type Outdomain<'s>;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>>;
}

/// A push-based op, specifying an Inflow domain/flow type.
pub trait PushOp: Op {
    type Inflow: Flow;
    type Indomain<'p>;

    type Feedback: Future;

    #[must_use]
    fn push<'p>(&mut self, item: Self::Indomain<'p>) -> Self::Feedback;
}
