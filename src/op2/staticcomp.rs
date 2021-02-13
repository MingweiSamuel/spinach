use std::pin::Pin;
use std::future::Future;
use std::task::{ Context, Poll };

use super::op::*;
use super::MoveNext;

pub struct StaticComp<I: PullOp, O: PushOp<Domain = I::Codomain>> {
    pull: I,
    push: O,
}

impl<I: PullOp, O: PushOp<Domain = I::Codomain>> StaticComp<I, O> {
    pub fn new(pull: I, push: O) -> Self {
        StaticComp {
            pull: pull,
            push: push,
        }
    }
}
impl<I: MovePullOp, O: MovePushOp<Domain = I::Codomain>> StaticComp<I, O> {
    pub async fn run_move(&mut self) {
        while let Some(item) = MoveNext::new(&mut self.pull).await {
            self.push.push(item).await;
            // TODO handle the feedback.
        }
    }
}
impl<I: RefPullOp, O: RefPushOp<Domain = I::Codomain>> StaticComp<I, O> {
    pub async fn run_ref(&mut self) {
        while let Some(_feedback) = RefStaticCompFuture::new(self).await {
            // TODO: handle the feedback.
        }
    }
}


struct RefStaticCompFuture<'a, I, O>
where
    I: RefPullOp,
    O: RefPushOp<Domain = I::Codomain>,
{
    comp_op: &'a mut StaticComp<I, O>,
    push_fut: Option<Pin<Box<O::Feedback>>>,
}
impl<'a, I, O> RefStaticCompFuture<'a, I, O>
where
    I: RefPullOp,
    O: RefPushOp<Domain = I::Codomain>,
{
    pub fn new(comp_op: &'a mut StaticComp<I, O>) -> Self {
        RefStaticCompFuture {
            comp_op: comp_op,
            push_fut: None,
        }
    }
}
impl<'a, I, O> Future for RefStaticCompFuture<'a, I, O>
where
    I: RefPullOp,
    O: RefPushOp<Domain = I::Codomain>,
    Self: Unpin,
{
    type Output = Option<<O::Feedback as Future>::Output>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Get a future if none created.
        if this.push_fut.is_none() {
            if let Poll::Ready(opt_item) = this.comp_op.pull.poll_next(ctx) {
                match opt_item {
                    Some(item) => {
                        this.push_fut = Some(Box::pin(this.comp_op.push.push(item)));
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
            poll_out.map(|item| Some(item))
        }
        else {
            Poll::Pending
        }
    }
}
