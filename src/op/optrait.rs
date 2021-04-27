use std::task::{Context, Poll};

pub trait Op<'s> {
    /// The output element type of this op. Has GAT lifetime.
    type Outdomain: 's;
}

pub trait OpDelta<'s>: Op<'s> {
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>>;
}

pub trait OpValue<'s>: Op<'s> {
    fn poll_value(&'s self, ctx: &mut Context<'_>) -> Poll<Self::Outdomain>;
}
