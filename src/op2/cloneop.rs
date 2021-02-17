use std::task::{ Context, Poll };

use super::op::*;

pub struct CloneOp<O: Op> {
    op: O,
}
impl<O: Op> CloneOp<O> {
    pub fn new(op: O) -> Self {
        CloneOp {
            op: op,
        }
    }
}
impl<O: Op> Op for CloneOp<O> {}
impl<O: PullOp> PullOp for CloneOp<O> {
    type Outflow = O::Outflow;
    type Outdomain = O::Outdomain;
}
impl<O: PushOp> PushOp for CloneOp<O> {
    type Inflow = O::Inflow;
    type Indomain = O::Indomain;
}
impl<O: RefPullOp> MovePullOp for CloneOp<O>
where
    O::Outdomain: Clone,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        let polled = self.op.poll_next(ctx);
        polled.map(|opt| opt.map(|item| item.clone()))
    }
}
impl<O: MovePushOp> RefPushOp for CloneOp<O>
where
    O::Indomain: Clone,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: &Self::Indomain) -> Self::Feedback {
        self.op.push(item.clone())
    }
}
