use std::future::Future;
use std::task::{ Context, Poll };



pub trait Op {}

pub trait PullOp: Op {
    type Codomain;
}
pub trait PushOp: Op {
    type Domain;
}



pub trait MovePullOp: PullOp {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>>;
}

pub trait RefPullOp: PullOp {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Codomain>>;
}



pub trait RefPushOp: PushOp {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: &Self::Domain) -> Self::Feedback;
}

pub trait MovePushOp: PushOp {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: Self::Domain) -> Self::Feedback;
}
