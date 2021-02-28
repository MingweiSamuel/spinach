//! Computation nodes.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::{join_all, JoinAll};

use crate::flow::Rx;
use crate::op::*;

/// An async function which puts the current task to sleep.
/// Unlike [`tokio::task::yield_now`], this marks the current task as not ready, so it
/// will remain asleep until the task is awoken by an event.
pub async fn sleep_yield_now() {
    /// Yield implementation
    struct SleepYieldNow {
        yielded: bool,
    }

    impl Future for SleepYieldNow {
        type Output = ();

        fn poll(mut self: std::pin::Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<()> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                // cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    SleepYieldNow { yielded: false }.await
}

// /// Helper future struct for getting a value from [`MovePullOp`]s.
// pub struct MoveNext<'a, O: PullOp> {
//     op: &'a mut O,
// }
// impl<'a, O: PullOp> MoveNext<'a, O> {
//     pub fn new(op: &'a mut O) -> Self {
//         Self { op }
//     }
// }
// impl<T, O: PullOp> Future for MoveNext<'_, O>
// where
//     Self: Unpin,
//     for<'b> O: PullOp<Outdomain<'b> = T>,
// {
//     type Output = Option<T>;

//     fn poll(self: std::pin::Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
//         self.get_mut().op.poll_next(ctx)
//     }
// }



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
        while let Some(_feedback) =
            StaticCompFuture::new(&mut self.pull, &mut self.push).await
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

/// A computation node with a single pull end and a single push end.
pub struct StaticMoveComp<I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = T>,
{
    pull: I,
    push: O,
}

impl<I, O, T> StaticMoveComp<I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = T>,
{
    /// Create a StaticComp from PULL and PUSH ops.
    pub fn new(pull: I, push: O) -> Self {
        Self { pull, push }
    }
}
impl<I, O, T> StaticMoveComp<I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = T>,
{
    /// If PULL and PUSH deal with owned values.
    /// Continuously runs this Comp node. Never returns! Use `tick_moveop` instead.
    pub async fn run(mut self) -> ! {
        while let Some(_feedback) =
            StaticMoveCompFuture::new(&mut self.pull, &mut self.push).await
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
            StaticMoveCompFuture::new(&mut self.pull, &mut self.push).await
        {
            Some(feedback)
        }
        else {
            None
        }
    }
}

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
    pub async fn add_split(&mut self, push: O) -> Option<Vec<<O::Feedback as Future>::Output>> {
        self.pushes.push(push);
        self.tick().await
    }

    /// For cloneable owned values.
    /// Continuously runs this Comp node. Never returns! Use `tick_moveop` instead.
    pub async fn run(mut self) -> ! {
        while let Some(_feedback) =
            DynCompFuture::new(&mut self.pull, self.pushes.iter_mut().collect()).await
        {
            // TODO: handle the feedback.
        }
        panic!();
    }

    /// For cloneable owned values.
    /// Runs a single element from the pull side through all the push sides.
    pub async fn tick(&mut self) -> Option<Vec<<O::Feedback as Future>::Output>> {
        DynCompFuture::new(&mut self.pull, self.pushes.iter_mut().collect()).await
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
    push_fut: Option<Pin<Box<JoinAll<O::Feedback>>>>,
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

impl<'s, I, O> Future for DynCompFuture<'s, I, O>
where
    I: PullOp<Outflow = Rx>,
    for<'a> I::Outdomain<'a>: Copy,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = I::Outdomain<'a>>,
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

/// A computation node with a single pull and dynamically many push ends.
pub struct DynRefComp<I, O, T>
where
    for<'a> I: PullOp<Outflow = Rx, Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a T>,
{
    pull: I,
    pushes: Vec<O>,
}

impl<I, O, T> DynRefComp<I, O, T>
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

impl<I, O, T> DynRefComp<I, O, T>
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
        while let Some(_feedback) =
            DynRefCompFuture::new(&mut self.pull, self.pushes.iter_mut().collect()).await
        {
            // TODO: handle the feedback.
        }
        panic!();
    }

    /// For cloneable owned values.
    /// Runs a single element from the pull side through all the push sides.
    pub fn tick(&mut self) -> DynRefCompFuture<'_, I, O, T> {
        DynRefCompFuture::new(&mut self.pull, self.pushes.iter_mut().collect())
    }
}
/// Internal future for dealing with dynamic comp work.
pub struct DynRefCompFuture<'s, I, O, T>
where
    for<'a> I: PullOp<Outflow = Rx, Outdomain<'a> = &'a T>,
    for<'a> O: PushOp<Inflow = Rx, Indomain<'a> = &'a T>,
{
    pull: &'s mut I,
    pushes: Vec<&'s mut O>,
    push_fut: Option<Pin<Box<JoinAll<O::Feedback>>>>,
}

impl<'s, I, O, T> DynRefCompFuture<'s, I, O, T>
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

impl<'s, I, O, T> Future for DynRefCompFuture<'s, I, O, T>
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

/// Internal future for dealing with reference comp work.
struct StaticMoveCompFuture<'s, I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = T>,
{
    pull: &'s mut I,
    push: &'s mut O,
    push_fut: Option<Pin<Box<O::Feedback>>>,
}

impl<'s, I, O, T> StaticMoveCompFuture<'s, I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = T>,
{
    pub fn new(pull: &'s mut I, push: &'s mut O) -> Self {
        Self {
            pull,
            push,
            push_fut: None,
        }
    }
}

impl<'s, I, O, T> Future for StaticMoveCompFuture<'s, I, O, T>
where
    for<'a> I: PullOp<Outdomain<'a> = T>,
    for<'a> O: PushOp<Inflow = I::Outflow, Indomain<'a> = T>,
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

