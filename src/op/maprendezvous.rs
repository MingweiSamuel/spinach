use std::task::{Context, Poll};

use super::*;

use crate::flow::*;
use crate::func::RendezvousFn;

/// Map-Flatten op for owned->owned values.
pub struct MapFlattenMoveRendezvousOp<O: Op, F: RendezvousFn>
where
    F::Outdomain: IntoIterator,
{
    op: O,
    func: F,
    /// A value produced by func, to be iterated in pull-based pipes.
    out: Option<<<F as RendezvousFn>::Outdomain as IntoIterator>::IntoIter>,
}
impl<O: Op, F: RendezvousFn> MapFlattenMoveRendezvousOp<O, F>
where
    F::Outdomain: IntoIterator,
{
    pub fn new(op: O, func: F) -> Self {
        Self {
            op,
            func,
            out: None,
        }
    }
}
impl<O: Op, F: RendezvousFn> Op for MapFlattenMoveRendezvousOp<O, F> where F::Outdomain: IntoIterator {}

impl<O, F: RendezvousFn> PullOp for MapFlattenMoveRendezvousOp<O, F>
where
    for<'a> O: PullOp<Outflow = Df, Outdomain<'a> = (F::InDf, &'a F::InRx)>,
    F::Outdomain: IntoIterator,
{
    type Outflow = Df;
    type Outdomain<'s> = <F::Outdomain as IntoIterator>::Item;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
        match self.out.as_mut().and_then(|out| out.next()) {
            Some(item) => Poll::Ready(Some(item)),
            None => {
                match self.op.poll_next(ctx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(None) => Poll::Ready(None), // EOS
                    Poll::Ready(Some(item)) => {
                        let mut newout = self.func.call(item).into_iter();
                        if let Some(outitem) = newout.next() {
                            Poll::Ready(Some(outitem))
                        } else {
                            self.poll_next(ctx)
                        }
                    }
                }
            }
        }
    }
}
