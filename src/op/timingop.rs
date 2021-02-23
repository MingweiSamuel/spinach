use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use tokio::time::{self, Sleep};

use crate::merge::Merge;

use super::*;

pub struct BlockingIntervalOp<O: PullOp> {
    op: O,
    interval: Duration,
    sleep: Pin<Box<Sleep>>,
}
impl<T, O: PullOp<Outflow = Df<T>>> BlockingIntervalOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op,
            interval,
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}
impl<T, O: PullOp<Outflow = Df<T>>> Op for BlockingIntervalOp<O> {}
impl<T, O: PullOp<Outflow = Df<T>>> PullOp for BlockingIntervalOp<O> {
    type Outflow = Df<T>;
}
impl<T, O: MovePullOp<Outflow = Df<T>>> MovePullOp for BlockingIntervalOp<O> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        match self.sleep.as_mut().poll(ctx) {
            Poll::Ready(_) => {
                match self.op.poll_next(ctx) {
                    Poll::Ready(Some(item)) => {
                        // If item available, reset the timer.
                        self.sleep = Box::pin(time::sleep(self.interval));
                        Poll::Ready(Some(item))
                    }
                    other => other, // Forward Poll::Ready(None) and Poll::Pending.
                }
            }
            Poll::Pending => Poll::Pending, // Forward Poll::Pending (i.e. interval not ready yet).
        }
    }
}
impl<T, O: RefPullOp<Outflow = Df<T>>> RefPullOp for BlockingIntervalOp<O> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
        match self.sleep.as_mut().poll(ctx) {
            Poll::Ready(_) => {
                match self.op.poll_next(ctx) {
                    Poll::Ready(Some(item)) => {
                        // If item available, reset the timer.
                        self.sleep = Box::pin(time::sleep(self.interval));
                        Poll::Ready(Some(item))
                    }
                    other => other, // Forward Poll::Ready(None) and Poll::Pending.
                }
            }
            Poll::Pending => Poll::Pending, // Forward Poll::Pending (i.e. interval not ready yet).
        }
    }
}

pub struct LeakyIntervalOp<O: PullOp> {
    op: O,
    interval: Duration,
    sleep: Pin<Box<Sleep>>,
}
impl<F: Merge, O: PullOp<Outflow = Rx<F>>> LeakyIntervalOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op,
            interval,
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}
impl<F: Merge, O: PullOp<Outflow = Rx<F>>> Op for LeakyIntervalOp<O> {}
impl<F: Merge, O: PullOp<Outflow = Rx<F>>> PullOp for LeakyIntervalOp<O> {
    type Outflow = Rx<F>;
}
impl<F: Merge, O: MovePullOp<Outflow = Rx<F>>> MovePullOp for LeakyIntervalOp<O> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        match self.op.poll_next(ctx) {
            Poll::Ready(Some(item)) => match self.sleep.as_mut().poll(ctx) {
                Poll::Ready(_) => {
                    self.sleep = Box::pin(time::sleep(self.interval));
                    Poll::Ready(Some(item))
                }
                Poll::Pending => Poll::Pending,
            },
            other => other, // Forward Poll::Ready(None) and Poll::Pending.
        }
    }
}
impl<F: Merge, O: RefPullOp<Outflow = Rx<F>>> RefPullOp for LeakyIntervalOp<O> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<&<Self::Outflow as Flow>::Domain>> {
        match self.op.poll_next(ctx) {
            Poll::Ready(Some(item)) => match self.sleep.as_mut().poll(ctx) {
                Poll::Ready(_) => {
                    self.sleep = Box::pin(time::sleep(self.interval));
                    Poll::Ready(Some(item))
                }
                Poll::Pending => Poll::Pending,
            },
            other => other, // Forward Poll::Ready(None) and Poll::Pending.
        }
    }
}

pub struct BatchingOp<O: PullOp> {
    op: O,
    interval: Duration,
    buffer: VecDeque<<O::Outflow as Flow>::Domain>,
    sleep: Pin<Box<Sleep>>,
}
impl<T, O: PullOp<Outflow = Df<T>>> BatchingOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op,
            interval,
            buffer: VecDeque::new(),
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}
impl<T, O: PullOp<Outflow = Df<T>>> Op for BatchingOp<O> {}
impl<T, O: PullOp<Outflow = Df<T>>> PullOp for BatchingOp<O> {
    type Outflow = O::Outflow;
}
impl<T, O: MovePullOp<Outflow = Df<T>>> MovePullOp for BatchingOp<O> {
    fn poll_next(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
        // Pull an element from the upstream op, keeping track if EOS.
        let poll_state = match self.op.poll_next(ctx) {
            Poll::Ready(Some(item)) => {
                self.buffer.push_back(item);
                Poll::Pending
            }
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
            }
            Poll::Pending => poll_state, // Propegate EOS or pending.
        }
    }
}
