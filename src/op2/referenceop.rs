// use std::task::{ Context, Poll };

use super::op::*;

pub struct ReferenceOp<O: Op> {
    op: O,
}
impl<O: Op> ReferenceOp<O> {
    pub fn new(op: O) -> Self {
        ReferenceOp {
            op: op,
        }
    }
}
impl<O: Op> Op for ReferenceOp<O> {}
// impl<O: PullOp> PullOp for ReferenceOp<O> {
//     type Outflow = O::Outflow;
//     type Outdomain = O::Outdomain;
// }
impl<O: PushOp> PushOp for ReferenceOp<O> {
    type Inflow = O::Inflow;
    type Indomain = O::Indomain;
}
// impl<O: MovePullOp> RefPullOp for ReferenceOp<O>
// where
//     O::Outdomain: Clone,
// {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Outdomain>> {
//         let polled = self.op.poll_next(ctx);
//         polled.map(|opt| opt.as_ref())
//     }
// }
impl<O: RefPushOp> MovePushOp for ReferenceOp<O>
where
    O::Indomain: Clone,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: Self::Indomain) -> Self::Feedback {
        self.op.push(&item)
    }
}
