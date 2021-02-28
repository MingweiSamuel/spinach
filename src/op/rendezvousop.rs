// use std::task::{Context, Poll};

// use crate::flow::*;

// use super::*;

// /// 
// pub struct RendezvousOp<O: PullOp<Outflow = Df>, P: PullOp<Outflow = Rx>> {
//     op_df: O,
//     op_rx: P,
// }
// impl<O: PullOp<Outflow = Df>, P: PullOp<Outflow = Rx>> RendezvousOp<O, P> {
//     /// Receive from both OP0 and OP1, where priority is to pull from OP0.
//     pub fn new(op_df: O, op_rx: P) -> Self {
//         RendezvousOp { op_df, op_rx }
//     }
// }
// impl<O: PullOp, P: PullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> Op for RendezvousOp<O, P> {}
// impl<O: PullOp, P: PullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> PullOp
//     for RendezvousOp<O, P>
// {
//     type Outflow = O::Outflow;
//     type Outdomain = O::Outdomain;
// }
// impl<O: MovePullOp, P: MovePullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> MovePullOp
//     for RendezvousOp<O, P>
// {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
//         let poll0 = self.op0.poll_next(ctx);
//         if let Poll::Ready(Some(item)) = poll0 {
//             return Poll::Ready(Some(item));
//         }
//         let poll1 = self.op1.poll_next(ctx);
//         if let Poll::Ready(Some(item)) = poll1 {
//             return Poll::Ready(Some(item));
//         }
//         if poll0.is_ready() && poll1.is_ready() {
//             // Both EOS.
//             Poll::Ready(None)
//         } else {
//             Poll::Pending
//         }
//     }
// }
// impl<O: RefPullOp, P: RefPullOp<Outflow = O::Outflow, Outdomain = O::Outdomain>> RefPullOp
//     for RendezvousOp<O, P>
// {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Outdomain>> {
//         let poll0 = self.op0.poll_next(ctx);
//         if let Poll::Ready(Some(item)) = poll0 {
//             return Poll::Ready(Some(item));
//         }
//         let poll1 = self.op1.poll_next(ctx);
//         if let Poll::Ready(Some(item)) = poll1 {
//             return Poll::Ready(Some(item));
//         }
//         if poll0.is_ready() && poll1.is_ready() {
//             // Both EOS.
//             Poll::Ready(None)
//         } else {
//             Poll::Pending
//         }
//     }
// }
