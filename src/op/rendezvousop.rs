use std::task::{Context, Poll};

use crate::flow::*;

use super::*;

/// An op which rendezvous dataflow values to a reactive value.
pub struct RendezvousOp<O: PullOp<Outflow = Df>, P: PullOp<Outflow = Rx>> {
    op_df: O,
    op_rx: P,
}

impl<O: PullOp<Outflow = Df>, P: PullOp<Outflow = Rx>> RendezvousOp<O, P> {
    pub fn new(op_df: O, op_rx: P) -> Self {
        Self { op_df, op_rx }
    }
}

impl<O: PullOp<Outflow = Df>, P: PullOp<Outflow = Rx>> Op for RendezvousOp<O, P> {}

impl<O: PullOp<Outflow = Df>, P: PullOp<Outflow = Rx>> PullOp for RendezvousOp<O, P>
{
    type Outflow = O::Outflow;
    type Outdomain<'s> = (O::Outdomain<'s>, P::Outdomain<'s>);

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
        // First get an RX value.
        let rx_poll = self.op_rx.poll_next(ctx);
        match rx_poll {
            Poll::Ready(Some(rx_item)) => {
                let df_poll = self.op_df.poll_next(ctx);
                match df_poll {
                    Poll::Ready(Some(df_item)) => Poll::Ready(Some((df_item, rx_item))),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
