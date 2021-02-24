use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::{join_all, JoinAll};

use crate::merge::Merge;

use super::*;

/// A computation node with a single pull end and a single push end.
pub struct StaticComp<I: PullOp, O: PushOp<Inflow = I::Outflow>> {
    pull: I,
    push: O,
}

impl<I: PullOp, O: PushOp<Inflow = I::Outflow>> StaticComp<I, O> {
    /// Create a StaticComp from PULL and PUSH ops.
    pub fn new(pull: I, push: O) -> Self {
        Self { pull, push }
    }
}
impl<I: MovePullOp, O: MovePushOp<Inflow = I::Outflow>> StaticComp<I, O> {
    /// If PULL and PUSH deal with owned values.
    /// Continuously runs this Comp node. Never returns! Use `tick_moveop` instead.
    pub async fn run_moveop(mut self) -> ! {
        while let Some(item) = MoveNext::new(&mut self.pull).await {
            self.push.push(item).await;
            // TODO handle the feedback.
        }
        panic!();
    }
    /// If PULL and PUSH deal with owned values.
    /// Runs a single element from the pull side through the push side.
    pub async fn tick_moveop(&mut self) -> Option<<O::Feedback as Future>::Output> {
        if let Some(item) = MoveNext::new(&mut self.pull).await {
            Some(self.push.push(item).await)
        } else {
            None
        }
    }
}
impl<I: RefPullOp, O: RefPushOp<Inflow = I::Outflow>> StaticComp<I, O> {
    /// If PULL and PUSH deal with reference values.
    /// Continuously runs this Comp node. Never returns! Use `tick_refop` instead.
    pub async fn run_refop(mut self) -> ! {
        while let Some(_feedback) = RefCompFuture::new(&mut self.pull, vec![&mut self.push]).await {
            // TODO: handle the feedback.
        }
        panic!();
    }
    /// If PULL and PUSH deal with reference values.
    /// Runs a single element from the pull side through the push side.
    pub async fn tick_refop(&mut self) -> Option<<O::Feedback as Future>::Output> {
        RefCompFuture::new(&mut self.pull, vec![&mut self.push])
            .await
            .map(|mut outs| outs.swap_remove(0))
    }
}

/// A computation node with a single pull and dynamically many push ends.
pub struct DynComp<F: Merge, I: PullOp<Outflow = Rx<F>>, O: PushOp<Inflow = Rx<F>>> {
    pull: I,
    pushes: Vec<O>,
}

impl<F: Merge, I: PullOp<Outflow = Rx<F>>, O: PushOp<Inflow = Rx<F>>> DynComp<F, I, O> {
    /// Create a DynComp from a pull end. Push ends can be added dynamically with `add_split`.
    pub fn new(pull: I) -> Self {
        Self {
            pull,
            pushes: vec![],
        }
    }
}
impl<F: Merge, I: MovePullOp<Outflow = Rx<F>>, O: MovePushOp<Inflow = Rx<F>>> DynComp<F, I, O>
where
    <I::Outflow as Flow>::Domain: Clone,
{
    /// For cloneable owned values.
    /// Adds a split off.
    pub async fn add_movesplit(&mut self, push: O) -> Option<Vec<<O::Feedback as Future>::Output>> {
        self.pushes.push(push);
        self.tick_moveop().await
    }

    /// For cloneable owned values.
    /// Continuously runs this Comp node. Never returns! Use `tick_moveop` instead.
    pub async fn run_moveop(mut self) -> ! {
        while let Some(item) = MoveNext::new(&mut self.pull).await {
            for push in &mut self.pushes {
                push.push(item.clone()).await;
                // TODO: one extra clone....
            }
            // // TODO handle the feedback.
        }
        panic!();
    }

    /// For cloneable owned values.
    /// Runs a single element from the pull side through all the push sides.
    pub async fn tick_moveop(&mut self) -> Option<Vec<<O::Feedback as Future>::Output>> {
        if let Some(item) = MoveNext::new(&mut self.pull).await {
            let futs = self.pushes.iter_mut().map(|push| push.push(item.clone())); // TODO: one extra clone...
            Some(join_all(futs).await)
        } else {
            None
        }
    }
}
impl<F: Merge, I: RefPullOp<Outflow = Rx<F>>, O: RefPushOp<Inflow = Rx<F>>> DynComp<F, I, O> {
    /// For reference values.
    /// Adds a split off.
    pub async fn add_refsplit(&mut self, push: O) -> Option<Vec<<O::Feedback as Future>::Output>> {
        self.pushes.push(push);
        self.tick_refop().await
    }

    /// For reference values.
    /// Continuously runs this Comp node. Never returns! Use `tick_moveop` instead.
    pub async fn run_refop(mut self) {
        while let Some(_feedback) =
            RefCompFuture::new(&mut self.pull, self.pushes.iter_mut().collect()).await
        {
            // TODO: handle the feedback.
        }
    }

    /// For reference values.
    /// Runs a single element from the pull side through all the push sides.
    pub async fn tick_refop(&mut self) -> Option<Vec<<O::Feedback as Future>::Output>> {
        RefCompFuture::new(&mut self.pull, self.pushes.iter_mut().collect()).await
    }
}

/// Internal future for dealing with reference comp work.
struct RefCompFuture<'a, I, O>
where
    I: RefPullOp,
    O: RefPushOp<Inflow = I::Outflow>,
{
    pull: &'a mut I,
    pushes: Vec<&'a mut O>,
    push_fut: Option<Pin<Box<JoinAll<O::Feedback>>>>,
}
impl<'a, I, O> RefCompFuture<'a, I, O>
where
    I: RefPullOp,
    O: RefPushOp<Inflow = I::Outflow>,
{
    pub fn new(pull: &'a mut I, pushes: Vec<&'a mut O>) -> Self {
        Self {
            pull,
            pushes,
            push_fut: None,
        }
    }
}
impl<'a, I, O> Future for RefCompFuture<'a, I, O>
where
    I: RefPullOp,
    O: RefPushOp<Inflow = I::Outflow>,
    Self: Unpin,
{
    type Output = Option<Vec<<O::Feedback as Future>::Output>>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Get a future if none created.
        if this.push_fut.is_none() {
            if let Poll::Ready(opt_item) = this.pull.poll_next(ctx) {
                match opt_item {
                    Some(item) => {
                        let futs = this.pushes.iter_mut().map(|push| push.push(item));
                        this.push_fut = Some(Box::pin(join_all(futs)));
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
        } else {
            Poll::Pending
        }
    }
}