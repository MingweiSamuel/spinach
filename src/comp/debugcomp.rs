use std::fmt::Debug;

use crate::lattice::LatticeRepr;
use crate::op::OpDelta;

use super::Next;

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
