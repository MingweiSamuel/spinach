use std::task::{ Context, Poll };

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
//     type Codomain = O::Codomain;
// }
impl<O: PushOp> PushOp for ReferenceOp<O> {
    type Inflow = O::Inflow;
    type Domain = O::Domain;
}
// impl<O: MovePullOp> RefPullOp for ReferenceOp<O>
// where
//     O::Codomain: Clone,
// {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Codomain>> {
//         let polled = self.op.poll_next(ctx);
//         polled.map(|opt| opt.as_ref())
//     }
// }
impl<O: RefPushOp> MovePushOp for ReferenceOp<O>
where
    O::Domain: Clone,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: Self::Domain) -> Self::Feedback {
        self.op.push(&item)
    }
}
