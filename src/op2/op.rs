use std::future::Future;
use std::task::{ Context, Poll };



pub trait Op {
    type Domain;
    type Codomain;
}



pub trait MovePullOp: Op {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>>;
}

pub trait RefPullOp: Op {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Codomain>>;
}



pub trait RefPushOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: &Self::Domain) -> Self::Feedback;
}

pub trait MovePushOp: Op {
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: Self::Domain) -> Self::Feedback;
}
