use std::future::Future;
use std::task::{ Context, Poll };

use super::Flow;


pub trait Op {}

pub trait PullOp<'slf>: Op {
    type Outflow<'a>: Flow;

    fn poll_next<'a>(&'slf mut self, ctx: &mut Context<'_>) -> Poll<Option<<Self::Outflow<'a> as Flow>::Domain>>;
}

pub trait PushOp<'slf>: Op {
    type Inflow<'a>: Flow;
    type Feedback<'a, 's>: Future;

    #[must_use]
    fn push<'a>(&'slf mut self, item: <Self::Inflow<'a> as Flow>::Domain) -> Self::Feedback<'a, 'slf>;
}
