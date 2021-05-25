use std::cell::{RefCell};
use std::future::Future;
use std::iter::IntoIterator;

use futures::future;

use crate::op::{OpDelta, OpValue, Splitter, SplitOp};
use crate::lattice::LatticeRepr;
use crate::lattice::setunion::SetUnion;

use super::{Comp, CompConnector, Next};

pub struct DynSplitComp<'a, O: OpValue + OpDelta, P, C>
where
    P: OpDelta,
    P::LatRepr: LatticeRepr<Lattice = SetUnion<C>>,
    <P::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = C>,
    C: CompConnector<SplitOp<'a, O>>,
{
    splitter: &'a Splitter<O>,
    pipe_op: P,

    splits: RefCell<Vec<C::Comp>>,
}

impl<'a, O: OpValue + OpDelta, P, C> DynSplitComp<'a, O, P, C>
where
    P: OpDelta,
    P::LatRepr: LatticeRepr<Lattice = SetUnion<C>>,
    <P::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = C>,
    C: CompConnector<SplitOp<'a, O>>,
{
    pub fn new(splitter: &'a Splitter<O>, pipe_op: P) -> Self {
        Self {
            splitter,
            pipe_op,

            splits: Default::default(),
        }
    }
}

impl<'a, O: OpValue + OpDelta, P, C> Comp for DynSplitComp<'a, O, P, C>
where
    P: OpDelta,
    P::LatRepr: LatticeRepr<Lattice = SetUnion<C>>,
    <P::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = C>,
    C: CompConnector<SplitOp<'a, O>>,
{
    type Error = <C::Comp as Comp>::Error;

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            // Join up any new splits.
            while let Some(hide_connectors) = (Next { op: &self.pipe_op }).await {
                for connector in hide_connectors.into_reveal() {
                    let new_split = connector.connect(self.splitter.add_split());
                    self.splits.borrow_mut().push(new_split);
                }
            }

            // Run all the ticks, remove any erroring comps.
            let tick_results = future::join_all(self.splits.borrow().iter().map(|comp| comp.tick())).await;
            {
                let mut splits = self.splits.borrow_mut();
                let mut index = 0;
                for tick_result in tick_results {
                    match tick_result {
                        Err(_) => {
                            splits.remove(index);
                        }
                        Ok(_) => {
                            index += 1;
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
