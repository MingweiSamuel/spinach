use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use tokio::time::{self, Sleep};

use crate::flow::*;

use super::*;

/// An op which releases individual values on a timer interval.
///
/// Values will be implicitly buffered by stalling the pipeline,
/// so this can be considered "blocking".
pub struct BlockingIntervalOp<O: PullOp> {
    op: O,
    interval: Duration,
    sleep: Pin<Box<Sleep>>,
}
impl<T, O: PullOp<Outflow = Df, Outdomain = T>> BlockingIntervalOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op,
            interval,
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}
impl<T, O: PullOp<Outflow = Df, Outdomain = T>> Op for BlockingIntervalOp<O> {}
impl<T, O: PullOp<Outflow = Df, Outdomain = T>> PullOp for BlockingIntervalOp<O> {
    type Outflow = Df;
    type Outdomain = T;
}
impl<T, O: MovePullOp<Outflow = Df, Outdomain = T>> MovePullOp for BlockingIntervalOp<O> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
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
impl<T, O: RefPullOp<Outflow = Df, Outdomain = T>> RefPullOp for BlockingIntervalOp<O> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Outdomain>> {
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

/// An op which releases individual values on a timer interval.
///
/// If the timer is not ready and a value is produced, the value will be dropped.
/// Therefore, this only applies to [`Rx`] flows.
pub struct LeakyIntervalOp<O: PullOp> {
    op: O,
    interval: Duration,
    sleep: Pin<Box<Sleep>>,
}
impl<T, O: PullOp<Outflow = Rx, Outdomain = T>> LeakyIntervalOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op,
            interval,
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}
impl<T, O: PullOp<Outflow = Rx, Outdomain = T>> Op for LeakyIntervalOp<O> {}
impl<T, O: PullOp<Outflow = Rx, Outdomain = T>> PullOp for LeakyIntervalOp<O> {
    type Outflow = Rx;
    type Outdomain = T;
}
impl<T, O: MovePullOp<Outflow = Rx, Outdomain = T>> MovePullOp for LeakyIntervalOp<O> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
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
impl<T, O: RefPullOp<Outflow = Rx, Outdomain = T>> RefPullOp for LeakyIntervalOp<O> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Outdomain>> {
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

/// An op which releases batches of values on a timer interval.
///
/// Values are buffered in a queue. Once an interval is reached all buffered
/// values will become available at once. In this sense it is non-blocking.
pub struct BatchingOp<O: PullOp> {
    op: O,
    interval: Duration,
    buffer: VecDeque<O::Outdomain>,
    sleep: Pin<Box<Sleep>>,
}
impl<T, O: PullOp<Outflow = Df, Outdomain = T>> BatchingOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op,
            interval,
            buffer: VecDeque::new(),
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}
impl<T, O: PullOp<Outflow = Df, Outdomain = T>> Op for BatchingOp<O> {}
impl<T, O: PullOp<Outflow = Df, Outdomain = T>> PullOp for BatchingOp<O> {
    type Outflow = O::Outflow;
    type Outdomain = O::Outdomain;
}
impl<T, O: MovePullOp<Outflow = Df, Outdomain = T>> MovePullOp for BatchingOp<O> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
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
