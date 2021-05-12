use std::future::{Future};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::fmt::Debug;

use crate::lattice::LatticeRepr;
use crate::op::OpDelta;
use crate::hide::{Hide, Delta};

struct Next<'s, O: OpDelta> {
    op: &'s O,
}

impl<O: OpDelta> Future for Next<'_, O> {
    type Output = Option<Hide<Delta, O::LatRepr>>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.op.poll_delta(ctx)
    }
}

pub struct DebugComp<O: OpDelta>
where
    <O::LatRepr as LatticeRepr>::Repr: Debug,
{
    op: O,
}

impl<O: OpDelta> DebugComp<O>
where
    <O::LatRepr as LatticeRepr>::Repr: Debug,
{
    pub fn new(op: O) -> Self {
        Self { op }
    }

    pub async fn tick(&self) -> Result<(), ()> {
        if let Some(hide) = (Next { op: &self.op }).await {
            println!("{:?}", hide.into_reveal());
            Ok(())
        }
        else {
            Err(())
        }
    }

    pub async fn run(&self) -> Result<!, ()> {
        loop {
            self.tick().await?;
        }
    }
}
