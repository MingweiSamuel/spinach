use std::fmt::Debug;
use std::future::Future;

use crate::lattice::LatticeRepr;
use crate::op::OpDelta;

use super::{Comp, Next};

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
}

impl<O: OpDelta> Comp for DebugComp<O>
where
    <O::LatRepr as LatticeRepr>::Repr: Debug,
{
    type Error = ();

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            if let Some(hide) = (Next { op: &self.op }).await {
                println!("{:?}", hide.into_reveal());
                Ok(())
            }
            else {
                Err(())
            }
        }
    }
}
