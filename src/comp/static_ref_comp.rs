use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::op::*;

/// A computation node with a single pull end and a single push end.
pub struct StaticRefComp<I, O, T: ?Sized>
where
    for<'a> I: PullOp<Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = &'a T>,
{
    pull: I,
    push: O,
}

impl<I, O, T: ?Sized> StaticRefComp<I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = &'a T>,
{
    /// Create a StaticComp from PULL and PUSH ops.
    pub fn new(pull: I, push: O) -> Self {
        StaticRefComp { pull, push }
    }
}
impl<I, O, T: ?Sized> StaticRefComp<I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = &'a T>,
{
    /// If PULL and PUSH deal with owned values.
    /// Continuously runs this Comp node. Never returns! Use `tick_moveop` instead.
    pub async fn run(mut self) -> ! {
        loop {
            if let Some(_feedback) = self.tick().await {
                // TODO: handle the feedback.
            }
        }
    }

    /// If PULL and PUSH deal with owned values.
    /// Runs a single element from the pull side through the push side.
    pub fn tick(&mut self) -> StaticMoveCompFuture<'_, I, O, T> {
        StaticMoveCompFuture::new(&mut self.pull, &mut self.push)
    }
}

/// Internal future for dealing with static move comp work.
pub struct StaticMoveCompFuture<'s, I, O, T: ?Sized>
where
    for<'a> I: PullOp<Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = &'a T>,
{
    pull: &'s mut I,
    push: &'s mut O,
    push_fut: Option<Pin<Box<O::Feedback>>>,
}

impl<'s, I, O, T: ?Sized> StaticMoveCompFuture<'s, I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = &'a T>,
{
    pub fn new(pull: &'s mut I, push: &'s mut O) -> Self {
        Self {
            pull,
            push,
            push_fut: None,
        }
    }
}

impl<'s, I, O, T: ?Sized> Future for StaticMoveCompFuture<'s, I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = &'a T>,
    Self: Unpin,
{
    type Output = Option<<O::Feedback as Future>::Output>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Get a future if none created.
        if this.push_fut.is_none() {
            if let Poll::Ready(opt_item) = this.pull.poll_next(ctx) {
                match opt_item {
                    Some(item) => {
                        let fut = this.push.push(item);
                        this.push_fut = Some(Box::pin(fut));
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
