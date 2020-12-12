use std::pin::Pin;
use std::task::{ Context, Poll };

use crate::merge::Merge;


pub trait Op {
    type Item;

    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>>;
}

pub trait RefOp {
    type Item;

    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<&Self::Item>>;
}



pub struct CloneOp<St: RefOp>
where
    St::Item: Clone,
{
    stream: St,
}
impl<St: RefOp> CloneOp<St>
where
    St::Item: Clone,
{
    pub fn new(stream: St) -> Self {
        Self {
            stream: stream,
        }
    }
}
impl<St: RefOp> Op for CloneOp<St>
where
    St::Item: Clone,
{
    type Item = St::Item;

    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.poll_next(ctx)
            .map(|opt| opt.map(|x| x.clone()))
    }
}



pub struct LatticeOp<St: Op, F: Merge<Domain = St::Item>> {
    stream: St,
    state: F::Domain,
}
impl<St: Op, F: Merge<Domain = St::Item>> LatticeOp<St, F> {
    pub fn new(stream: St, bottom: F::Domain) -> Self {
        Self {
            stream: stream,
            state: bottom,
        }
    }
}
impl<St: Op, F: Merge<Domain = St::Item>> RefOp for LatticeOp<St, F> {
    type Item = St::Item;

    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Item>> {
        if let Poll::Ready(Some(delta)) = self.stream.poll_next(ctx) {
            F::merge_in(&mut self.state, delta);
        }
        Poll::Ready(Some(&self.state))
    }
}
