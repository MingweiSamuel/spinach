use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{ Context, Poll };
use std::time::Duration;

use tokio::time::{ self, Sleep };

use super::op::*;
use super::flow::Flow;


pub struct RateLimitOp<O: PullOp> {
    op: O,
    interval: Duration,
    sleep: Pin<Box<Sleep>>,
}
impl<O: PullOp> RateLimitOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op: op,
            interval: interval,
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}
impl<O: PullOp> Op for RateLimitOp<O> {}
impl<O: PullOp> PullOp for RateLimitOp<O> {
    type Outflow = O::Outflow;
}
impl<O: MovePullOp> MovePullOp for RateLimitOp<O> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        match self.sleep.as_mut().poll(ctx) {
            Poll::Ready(_) => {
                match self.op.poll_next(ctx) {
                    Poll::Ready(Some(item)) => {
                        // If item available, reset the timer.
                        self.sleep = Box::pin(time::sleep(self.interval));
                        Poll::Ready(Some(item))
                    },
                    other => other, // Forward Poll::Ready(None) and Poll::Pending.
                }
            },
            Poll::Pending => Poll::Pending, // Forward Poll::Pending (i.e. interval not ready yet).
        }
    }
}
impl<O: RefPullOp> RefPullOp for RateLimitOp<O>
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
        match self.sleep.as_mut().poll(ctx) {
            Poll::Ready(_) => {
                match self.op.poll_next(ctx) {
                    Poll::Ready(Some(item)) => {
                        // If item available, reset the timer.
                        self.sleep = Box::pin(time::sleep(self.interval));
                        Poll::Ready(Some(item))
                    },
                    other => other, // Forward Poll::Ready(None) and Poll::Pending.
                }
            },
            Poll::Pending => Poll::Pending, // Forward Poll::Pending (i.e. interval not ready yet).
        }
    }
}





pub struct BatchingOp<O: PullOp> {
    op: O,
    interval: Duration,
    buffer: VecDeque<<O::Outflow as Flow>::Domain>,
    sleep: Pin<Box<Sleep>>,
}
impl<O: PullOp> BatchingOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op: op,
            interval: interval,
            buffer: VecDeque::new(),
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}
impl<O: PullOp> Op for BatchingOp<O> {}
impl<O: PullOp> PullOp for BatchingOp<O> {
    type Outflow = O::Outflow;
}
impl<O: MovePullOp> MovePullOp for BatchingOp<O> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        // Pull an element from the upstream op, keeping track if EOS.
        let poll_state = match self.op.poll_next(ctx) {
            Poll::Ready(Some(item)) => {
                self.buffer.push_back(item);
                Poll::Pending
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        };

        match self.sleep.as_mut().poll(ctx) {
            Poll::Ready(_) => {
                // If timer is ready.
                match self.buffer.pop_front() {
                    // Get an item from the buffer.
                    Some(item) => Poll::Ready(Some(item)),
                    // If the buffer is empty, reset the timer.
                    None => {
                        self.sleep = Box::pin(time::sleep(self.interval));
                        poll_state // Propegate EOS or pending.
                    }
                }
            },
            Poll::Pending => poll_state, // Propegate EOS or pending.
        }
    }
}
