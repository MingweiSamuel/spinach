use std::pin::Pin;
use std::future::Future;
use std::task::{ Context, Poll };

use futures::future::{ join_all, JoinAll };

use super::op::*;
use super::types::*;
// use super::MoveNext;

pub struct DynSplitComp<I: PullOp<Outflow = RX>, O: PushOp<Inflow = RX, Domain = I::Outdomain>> {
    pull: I,
    pushes: Vec<O>,
}

impl<I: PullOp<Outflow = RX>, O: PushOp<Inflow = RX, Domain = I::Outdomain>> DynSplitComp<I, O> {
    pub fn new(pull: I) -> Self {
        DynSplitComp {
            pull: pull,
            pushes: Default::default(),
        }
    }
}
// impl<I: MovePullOp<Outflow = RX>, O: MovePushOp<Inflow = RX, Domain = I::Outdomain>> DynSplitComp<I, O> {
//     pub async fn run_move(&mut self) {
//         while let Some(item) = MoveNext::new(&mut self.pull).await {
//             self.push.push(item).await;
//             // TODO handle the feedback.
//         }
//     }
// }
impl<I: RefPullOp<Outflow = RX>, O: RefPushOp<Inflow = RX, Domain = I::Outdomain>> DynSplitComp<I, O> {
    pub async fn run_ref(&mut self) {
        while let Some(_feedback) = RefStaticCompFuture::new(self).await {
            // TODO: handle the feedback.
        }
    }
}


struct RefStaticCompFuture<'a, I, O>
where
    I: RefPullOp<Outflow = RX>,
    O: RefPushOp<Inflow = RX, Domain = I::Outdomain>,
{
    comp_op: &'a mut DynSplitComp<I, O>,
    push_fut: Option<Pin<Box<JoinAll<O::Feedback>>>>,
}
impl<'a, I, O> RefStaticCompFuture<'a, I, O>
where
    I: RefPullOp<Outflow = RX>,
    O: RefPushOp<Inflow = RX, Domain = I::Outdomain>,
{
    pub fn new(comp_op: &'a mut DynSplitComp<I, O>) -> Self {
        RefStaticCompFuture {
            comp_op: comp_op,
            push_fut: None,
        }
    }
}
impl<'a, I, O> Future for RefStaticCompFuture<'a, I, O>
where
    I: RefPullOp<Outflow = RX>,
    O: RefPushOp<Inflow = RX, Domain = I::Outdomain>,
    Self: Unpin,
{
    type Output = Option<Vec<<O::Feedback as Future>::Output>>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Get a future if none created.
        if this.push_fut.is_none() {
            if let Poll::Ready(opt_item) = this.comp_op.pull.poll_next(ctx) {
                match opt_item {
                    Some(item) => {
                        let push_futs = this.comp_op.pushes
                            .iter_mut()
                            .map(|push| push.push(item));
                        this.push_fut = Some(Box::pin(join_all(push_futs)));
                    }
                    None => {
                        return Poll::Ready(None);
                    }
                }
            }
        }

        // Poll the future if it's available.
        if let Some(push_fut) = &mut this.push_fut {
            let poll_out = push_fut.as_mut().poll(ctx);
            if poll_out.is_ready() {
                this.push_fut = None;
            }
            poll_out.map(|items| Some(items))
        }
        else {
            Poll::Pending
        }
    }
}
