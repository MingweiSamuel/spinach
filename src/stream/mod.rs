use std::pin::Pin;
use std::task::{ Context, Poll };

// use futures_core::stream::Stream;
use pin_project::pin_project;

use crate::merge::Merge;


pub trait Op {
    type Item;

    fn poll_next(
        self: &mut Self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>>;
}


pub trait RefOp {
    type Item;

    fn poll_next(
        self: &mut Self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<&Self::Item>>;
}

pub struct LatticeState<St: Op, F: Merge<Domain = St::Item>> {
    stream: St,
    state: F::Domain,
}
impl<St: Op, F: Merge<Domain = St::Item>> LatticeState<St, F> {
    pub fn new(stream: St, bottom: F::Domain) -> Self {
        Self {
            stream: stream,
            state: bottom,
        }
    }
}
impl<St: Op, F: Merge<Domain = St::Item>> RefOp for LatticeState<St, F> {
    type Item = St::Item;

    fn poll_next(self: &mut Self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Item>> {
        if let Poll::Ready(Some(delta)) = self.stream.poll_next(ctx) {
            F::merge_in(&mut self.state, delta);
        }
        Poll::Ready(Some(&self.state))
    }
}
