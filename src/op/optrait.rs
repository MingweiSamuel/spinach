use std::task::{Context, Poll};

pub enum FlowType {
    Delta,
    Value,
}

/// A pull-based op, specifying an Outflow domain/flow type.
pub trait Op<'s> {
    /// The output element type of this op. Has GAT lifetime.
    type Outdomain;

    fn poll_value(&'s self, flow_type: FlowType, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>>;
}
