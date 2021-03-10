use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::{join_all, JoinAll};

use crate::flow::Rx;
use crate::op::*;

/// A computation node with a single pull and dynamically many push ends.
pub struct DynComp<I, O>
where
    I: PullOp<Outflow = Rx>,
    for<'a> I::Outdomain<'a>: Copy,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = I::Outdomain<'a>>,
{
    pull: I,
    pushes: Vec<O>,
}

impl<I, O> DynComp<I, O>
where
    I: PullOp<Outflow = Rx>,
    for<'a> I::Outdomain<'a>: Copy,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = I::Outdomain<'a>>,
{
    /// Create a DynComp from a pull end. Push ends can be added dynamically with `add_split`.
    pub fn new(pull: I) -> Self {
        Self {
            pull,
            pushes: vec![],
        }
    }
}

impl<I, O> DynComp<I, O>
where
    I: PullOp<Outflow = Rx>,
    for<'a> I::Outdomain<'a>: Copy,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = I::Outdomain<'a>>,
{
    /// For cloneable owned values.
    /// Adds a split off.
    pub async fn add_split(&mut self, push: O) -> Option<Vec<<O::Feedback<'_> as Future>::Output>> {
        self.pushes.push(push);
        self.tick().await
    }

    /// For cloneable owned values.
    /// Continuously runs this Comp node. Never returns! Use `tick_moveop` instead.
    pub async fn run(mut self) -> ! {
        loop {
            if let Some(_feedback) = self.tick().await {
                // TODO: handle the feedback.
            }
        }
    }

    /// For cloneable owned values.
    /// Runs a single element from the pull side through all the push sides.
    pub async fn tick(&mut self) -> Option<Vec<<O::Feedback<'_> as Future>::Output>> {
        let fut = DynCompFuture::new(&mut self.pull, self.pushes.iter_mut().collect());
        (&fut).await
    }
}

/// Internal future for dealing with dynamic comp work.
struct DynCompFuture<'s, I, O>
where
    I: PullOp<Outflow = Rx>,
    for<'a> I::Outdomain<'a>: Copy,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = I::Outdomain<'a>>,
{
    pull: &'s mut I,
    pushes: Vec<&'s mut O>,
    push_fut: Option<Pin<Box<JoinAll<O::Feedback<'s>>>>>,
}

impl<'s, I, O> DynCompFuture<'s, I, O>
where
    I: PullOp<Outflow = Rx>,
    for<'a> I::Outdomain<'a>: Copy,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = I::Outdomain<'a>>,
{
    pub fn new(pull: &'s mut I, pushes: Vec<&'s mut O>) -> Self {
        Self {
            pull,
            pushes,
            push_fut: None,
        }
    }
}

impl<'s, I, O> Future for &'s DynCompFuture<'s, I, O>
where
    I: PullOp<Outflow = Rx>,
    for<'a> I::Outdomain<'a>: Copy,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = I::Outdomain<'a>>,
    Self: Unpin,
{
    type Output = Option<Vec<<O::Feedback<'s> as Future>::Output>>;

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
