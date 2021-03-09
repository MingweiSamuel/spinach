use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::{join_all, JoinAll};

use crate::flow::Rx;
use crate::op::*;
/// A computation node with a single pull and dynamically many push ends.
pub struct DynRefComp<I, O, T: ?Sized>
where
    for<'a> I: PullOp<Outflow = Rx, Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a T>,
{
    pull: I,
    pushes: Vec<O>,
}

impl<I, O, T: ?Sized> DynRefComp<I, O, T>
where
    for<'a> I: PullOp<Outflow = Rx, Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a T>,
{
    /// Create a DynComp from a pull end. Push ends can be added dynamically with `add_split`.
    pub fn new(pull: I) -> Self {
        Self {
            pull,
            pushes: vec![],
        }
    }
}

impl<I, O, T: ?Sized> DynRefComp<I, O, T>
where
    for<'a> I: PullOp<Outflow = Rx, Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a T>,
{
    /// For cloneable owned values.
    /// Adds a split off.
    pub fn add_split(&mut self, push: O) -> DynRefCompFuture<'_, I, O, T> {
        self.pushes.push(push);
        self.tick()
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
    pub fn tick(&mut self) -> DynRefCompFuture<'_, I, O, T> {
        DynRefCompFuture::new(&mut self.pull, self.pushes.iter_mut().collect())
    }
}

/// Internal future for dealing with dynamic comp work.
pub struct DynRefCompFuture<'s, I, O, T: ?Sized>
where
    for<'a> I: PullOp<Outflow = Rx, Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a T>,
{
    pull: &'s mut I,
    pushes: Vec<&'s mut O>,
    push_fut: Option<Pin<Box<JoinAll<O::Feedback>>>>,
}

impl<'s, I, O, T: ?Sized> DynRefCompFuture<'s, I, O, T>
where
    for<'a> I: PullOp<Outflow = Rx, Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a T>,
{
    pub fn new(pull: &'s mut I, pushes: Vec<&'s mut O>) -> Self {
        Self {
            pull,
            pushes,
            push_fut: None,
        }
    }
}

impl<'s, I, O, T: ?Sized> Future for DynRefCompFuture<'s, I, O, T>
where
    for<'a> I: PullOp<Outflow = Rx, Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a T>,
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
