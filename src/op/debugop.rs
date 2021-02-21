use std::fmt::Debug;
use std::task::{ Context, Poll };

use super::op::*;
use super::flow::Flow;


pub struct DebugOp<O: Op> {
    op: O,
    tag: &'static str,
}

impl<O: Op> DebugOp<O> {
    pub fn new(op: O, tag: &'static str) -> Self {
        Self {
            op: op,
            tag: tag,
        }
    }
}

impl<O: Op> Op for DebugOp<O> {}

impl<'slf, O: PullOp<'slf>> PullOp<'slf> for DebugOp<O>
where
    for<'a> <<O as PullOp<'slf>>::Outflow<'a> as Flow>::Domain: Debug,
{
    type Outflow<'a> = O::Outflow<'a>;

    fn poll_next<'a>(&'slf mut self, ctx: &mut Context<'_>) -> Poll<Option<<Self::Outflow<'a> as Flow>::Domain>> {
        let polled = self.op.poll_next(ctx);
        match &polled {
            Poll::Ready(Some(item)) => println!("{}: {:?}", self.tag, item),
            _ => {},
        }
        polled
    }
}

impl<'slf, O: PushOp<'slf>> PushOp<'slf> for DebugOp<O>
where
    for<'a> <<O as PushOp<'slf>>::Inflow<'a> as Flow>::Domain: Debug,
{
    type Inflow<'a> = O::Inflow<'a>;
    type Feedback<'a, 's> = O::Feedback<'a, 's>;

    #[must_use]
    fn push<'a>(&'slf mut self, item: <Self::Inflow<'a> as Flow>::Domain) -> Self::Feedback<'a, 'slf> {
        println!("{}: {:?}", self.tag, item);
        self.op.push(item)
    }
}
