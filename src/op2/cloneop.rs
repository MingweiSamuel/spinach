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
    type Codomain = O::Codomain;
}
impl<O: PushOp> PushOp for CloneOp<O> {
    type Domain = O::Domain;
}
impl<O: RefPullOp> MovePullOp for CloneOp<O>
where
    O::Codomain: Clone,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>> {
        let polled = self.op.poll_next(ctx);
        polled.map(|opt| opt.map(|item| item.clone()))
    }
}
impl<O: MovePushOp> RefPushOp for CloneOp<O>
where
    O::Domain: Clone,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: &Self::Domain) -> Self::Feedback {
        self.op.push(item.clone())
    }
}
