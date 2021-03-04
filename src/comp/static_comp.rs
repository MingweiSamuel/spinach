use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::op::*;

/// A computation node with a single pull end and a single push end.
pub struct StaticComp<I, O>
where
    I: PullOp,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = I::Outdomain<'a>>,
{
    pull: I,
    push: O,
}

impl<I, O> StaticComp<I, O>
where
    I: PullOp,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = I::Outdomain<'a>>,
{
    /// Create a StaticComp from PULL and PUSH ops.
    pub fn new(pull: I, push: O) -> Self {
        Self { pull, push }
    }
}
impl<I, O> StaticComp<I, O>
where
    I: PullOp,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = I::Outdomain<'a>>,
{
    /// If PULL and PUSH deal with owned values.
    /// Continuously runs this Comp node. Never returns! Use `tick_moveop` instead.
    pub async fn run(mut self) -> ! {
        while let Some(_feedback) = self.tick().await
        {
            // TODO: handle the feedback.
        }
        panic!();
    }

    /// If PULL and PUSH deal with owned values.
    /// Runs a single element from the pull side through the push side.
    pub async fn tick(&mut self) -> Option<<O::Feedback as Future>::Output> {
        #[allow(clippy::manual_map)]
        if let Some(feedback) =
            StaticCompFuture::new(&mut self.pull, &mut self.push).await
        {
            Some(feedback)
        }
        else {
            None
        }
    }
}

/// Internal future for dealing with reference comp work.
struct StaticCompFuture<'s, I, O>
where
    I: PullOp,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = I::Outdomain<'a>>,
{
    pull: &'s mut I,
    push: &'s mut O,
    push_fut: Option<Pin<Box<O::Feedback>>>,
}

impl<'s, I, O> StaticCompFuture<'s, I, O>
where
    I: PullOp,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = I::Outdomain<'a>>,
{
    pub fn new(pull: &'s mut I, push: &'s mut O) -> Self {
        Self {
            pull,
            push,
            push_fut: None,
        }
    }
}

impl<'s, I, O> Future for StaticCompFuture<'s, I, O>
where
    I: PullOp,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = I::Outdomain<'a>>,
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
