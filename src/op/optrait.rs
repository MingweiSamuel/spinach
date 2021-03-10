use std::future::Future;
use std::task::{Context, Poll};

use crate::flow::*;

/// An empty trait indicating a struct can be used as an Op.
pub trait Op {}

/// A pull-based op, specifying an Outflow domain/flow type.
pub trait PullOp: Op {
    /// The output flow type of this op.
    type Outflow: Flow;
    /// The output element type of this op. Has GAT lifetime `'s`.
    type Outdomain<'s>;

    /// Polls a value if available, similar to the standard async Stream trait.
    /// Output has GAT lifetime parameter `Self::Outdomain<'s>`.
    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>>;
}

/// A push-based op, specifying an Inflow domain/flow type.
pub trait PushOp: Op {
    /// The input flow type of this op.
    type Inflow: Flow;
    /// The output element type of this op. Has GAT lifetime `'p`.
    type Indomain<'p>;

    /// The future returned by the `push()` method.
    type Feedback<'s>: Future;

    /// Pushes a value into this op. `item` has GAT lifetime paramer `Self::Indomain<'p>`.
    #[must_use]
    fn push<'s, 'p>(&'s mut self, item: Self::Indomain<'p>) -> Self::Feedback<'s>;
}
