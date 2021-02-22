// use std::fmt::Debug;
// use std::task::{ Context, Poll };

// use super::op::*;
// use super::flow::Flow;

// use crate::merge::Merge;


// pub struct ExplodeOp<O: Op, F: Merge>
// where
//     F::Domain: Clone
// {
//     op: O,
//     state: F::Domain,
// }
// impl<O: Op, F: Merge> ExplodeOp<O, F>
// where
//     F::Domain: Clone
// {
//     pub fn new(op: O, bottom: F::Domain) -> Self {
//         Self {
//             op: op,
//             state: bottom,
//         }
//     }
// }
// impl<O: Op, F: Merge> ExplodeOp<O, F>
// where
//     F::Domain: Clone + Default
// {
//     pub fn new_default(op: O) -> Self {
//         Self {
//             op: op,
//             state: Default::default(),
//         }
//     }
// }

// impl<O: Op, F: Merge> Op for ExplodeOp<O, F>
// where
//     F::Domain: Clone
// {}

// impl<O: PullOp, F: Merge> PullOp for DebugOp<O>
// where
//     F::Domain: Clone
// {
//     type Outflow = O::Outflow;
// }
// impl<O: PushOp> PushOp for DebugOp<O> {
//     type Inflow = O::Inflow;
// }
// impl<O: MovePullOp> MovePullOp for DebugOp<O>
// where
//     <O::Outflow as Flow>::Domain: Debug,
// {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
//         let polled = self.op.poll_next(ctx);
//         match &polled {
//             Poll::Ready(Some(item)) => println!("{}: {:?}", self.tag, item),
//             _ => (),
//         }
//         polled
//     }
// }
// impl<O: RefPullOp> RefPullOp for DebugOp<O>
// where
//     <O::Outflow as Flow>::Domain: Debug,
// {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
//         let polled = self.op.poll_next(ctx);
//         match &polled {
//             Poll::Ready(Some(item)) => println!("{}: {:?}", self.tag, item),
//             _ => (),
//         }
//         polled
//     }
// }
// impl<O: MovePushOp> MovePushOp for DebugOp<O>
// where
//     <O::Inflow as Flow>::Domain: Debug,
// {
//     type Feedback = O::Feedback;

//     fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
//         println!("{}: {:?}", self.tag, item);
//         self.op.push(item)
//     }
// }
// impl<O: RefPushOp> RefPushOp for DebugOp<O>
// where
//     <O::Inflow as Flow>::Domain: Debug,
// {
//     type Feedback = O::Feedback;

//     fn push(&mut self, item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback {
//         println!("{}: {:?}", self.tag, item);
//         self.op.push(item)
//     }
// }
