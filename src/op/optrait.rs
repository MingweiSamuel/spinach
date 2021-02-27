use std::future::Future;
use std::task::{Context, Poll};

use crate::flow::*;

/// An empty trait indicating a struct can be used as an Op.
pub trait Op {}

/// A pull-based op, specifying an Outflow domain/flow type.
pub trait PullOp: Op {
    type Outflow: Flow;
}

/// A push-based op, specifying an Inflow domain/flow type.
pub trait PushOp: Op {
    type Inflow: Flow;
    // type Indomain;
}

/// Pull-based op for owned values.
pub trait MovePullOp: PullOp {
    fn poll_next(&mut self, ctx: &mut Context<'_>)
        -> Poll<Option<<Self::Outflow as Flow>::Domain>>;
}

/// Pull-based op for reference values.
pub trait RefPullOp: PullOp {
    fn poll_next<'a>(
        &'a mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::RefDomain<'a>>>;
}

/// Push-based op for owned values.
pub trait MovePushOp: PushOp {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback;
}

/// Push-based op for reference values.
pub trait RefPushOp: PushOp {
    type Feedback: Future;

    #[must_use]
    fn push<'a>(&mut self, item: <Self::Inflow as Flow>::RefDomain<'a>) -> Self::Feedback;
}
