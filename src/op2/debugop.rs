use std::fmt::Debug;
use std::task::{ Context, Poll };

use super::op::*;


pub struct DebugOp<O: Op> {
    tag: &'static str,
    op: O,
}
impl<O: Op> DebugOp<O> {
    pub fn new(tag: &'static str, op: O) -> Self {
        DebugOp {
            tag: tag,
            op: op,
        }
    }
}
impl<O: Op> Op for DebugOp<O> {}
impl<O: PullOp> PullOp for DebugOp<O> {
    type Outflow = O::Outflow;
    type Outdomain = O::Outdomain;
}
impl<O: PushOp> PushOp for DebugOp<O> {
    type Inflow = O::Inflow;
    type Indomain = O::Indomain;
}
impl<O: MovePullOp> MovePullOp for DebugOp<O>
where
    O::Outdomain: Debug,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        let polled = self.op.poll_next(ctx);
        match &polled {
            Poll::Ready(Some(item)) => println!("{}: {:?}", self.tag, item),
            _ => (),
        }
        polled
    }
}
impl<O: RefPullOp> RefPullOp for DebugOp<O>
where
    O::Outdomain: Debug,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Outdomain>> {
        let polled = self.op.poll_next(ctx);
        match &polled {
            Poll::Ready(Some(item)) => println!("{}: {:?}", self.tag, item),
            _ => (),
        }
        polled
    }
}
impl<O: MovePushOp> MovePushOp for DebugOp<O>
where
    O::Indomain: Debug,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: Self::Indomain) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.op.push(item)
    }
}
impl<O: RefPushOp> RefPushOp for DebugOp<O>
where
    O::Indomain: Debug,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: &Self::Indomain) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.op.push(item)
    }
}
