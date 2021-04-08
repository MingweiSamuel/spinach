use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::{join_all, JoinAll};

use crate::lattice::Lattice;
use crate::flow::Df;
use crate::op::*;
/// A computation node with a single pull and dynamically many push ends.
pub struct LatticeComp<I, O, F: Lattice>
where
    for<'a> I: PullOp<Outflow = Df, Outdomain<'a> = F::Domain>,
    for<'a> O: PushOp<Inflow = Df, Indomain<'a> = &'a F::Domain>,
    F::Domain: Unpin,
{
    state: F::Domain,
    pull: I,
    pushes: Vec<O>,
}

impl<I, O, F: Lattice> LatticeComp<I, O, F>
where
    for<'a> I: PullOp<Outflow = Df, Outdomain<'a> = F::Domain>,
    for<'a> O: PushOp<Inflow = Df, Indomain<'a> = &'a F::Domain>,
    F::Domain: Unpin,
{
    /// Create a DynComp from a pull end. Push ends can be added dynamically with `add_split`.
    pub fn new(pull: I, bottom: F::Domain) -> Self {
        Self {
            state: bottom,
            pull,
            pushes: vec![],
        }
    }
}

impl<I, O, F: Lattice> LatticeComp<I, O, F>
where
    for<'a> I: PullOp<Outflow = Df, Outdomain<'a> = F::Domain>,
    for<'a> O: PushOp<Inflow = Df, Indomain<'a> = &'a F::Domain>,
    F::Domain: Unpin,
{
    /// For cloneable owned values.
    /// Adds a split off.
    pub async fn add_split(&mut self, mut push: O) -> Option<Vec<<O::Feedback as Future>::Output>> {
        push.push(&self.state).await;
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
    pub fn tick(&mut self) -> LatticeCompFuture<'_, I, O, F> {
        LatticeCompFuture::new(&mut self.state, &mut self.pull, self.pushes.iter_mut().collect())
    }
}

/// Internal future for dealing with dynamic comp work.
pub struct LatticeCompFuture<'s, I, O, F: Lattice>
where
    for<'a> I: PullOp<Outflow = Df, Outdomain<'a> = F::Domain>,
    for<'a> O: PushOp<Inflow = Df, Indomain<'a> = &'a F::Domain>,
{
    state: &'s mut F::Domain,
    pull: &'s mut I,
    pushes: Vec<&'s mut O>,
    push_fut: Option<Pin<Box<JoinAll<O::Feedback>>>>,
    item: Option<F::Domain>,
}

impl<'s, I, O, F: Lattice> LatticeCompFuture<'s, I, O, F>
where
    for<'a> I: PullOp<Outflow = Df, Outdomain<'a> = F::Domain>,
    for<'a> O: PushOp<Inflow = Df, Indomain<'a> = &'a F::Domain>,
{
    pub fn new(state: &'s mut F::Domain, pull: &'s mut I, pushes: Vec<&'s mut O>) -> Self {
        Self {
            state,
            pull,
            pushes,
            push_fut: None,
            item: None,
        }
    }
}

impl<'s, I, O, F: Lattice> Future for LatticeCompFuture<'s, I, O, F>
where
    for<'a> I: PullOp<Outflow = Df, Outdomain<'a> = F::Domain>,
    for<'a> O: PushOp<Inflow = Df, Indomain<'a> = &'a F::Domain>,
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
                        let futs = this.pushes.iter_mut().map(|push| push.push(&item));
                        this.push_fut = Some(Box::pin(join_all(futs)));
                        this.item = Some(item);
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
                F::merge_in(this.state, this.item.take().unwrap());
            }
            poll_out.map(|item| Some(item))
        } else {
            Poll::Pending
        }
    }
}
