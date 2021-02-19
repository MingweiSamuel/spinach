use std::future::Future;
use std::task::{ Context, Poll };

use super::Flow;


pub trait Op {}

pub trait PullOp: Op {
    type Outflow: Flow;
    // type Outdomain;
}
pub trait PushOp: Op {
    type Inflow: Flow;
    // type Indomain;
}



pub trait MovePullOp: PullOp {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<<Self::Outflow as Flow>::Domain>>;
}

pub trait RefPullOp: PullOp {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&<Self::Outflow as Flow>::Domain>>;
}



pub trait RefPushOp: PushOp {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback;
}

pub trait MovePushOp: PushOp {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback;
}
