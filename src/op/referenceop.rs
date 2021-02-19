// use std::task::{ Context, Poll };

use super::op::*;
use super::flow::Flow;

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
//     type Outdomain = <O::Outflow as Flow>::Domain;
// }
impl<O: PushOp> PushOp for ReferenceOp<O> {
    type Inflow = O::Inflow;
}
// impl<O: MovePullOp> RefPullOp for ReferenceOp<O>
// where
//     <O::Outflow as Flow>::Domain: Clone,
// {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
//         let polled = self.op.poll_next(ctx);
//         polled.map(|opt| opt.as_ref())
//     }
// }
impl<O: RefPushOp> MovePushOp for ReferenceOp<O>
where
    <O::Inflow as Flow>::Domain: Clone,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        self.op.push(&item)
    }
}
