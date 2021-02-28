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
pub struct BlockingIntervalOp<O: PullOp<Outflow = Df>> {
    op: O,
    interval: Duration,
    sleep: Pin<Box<Sleep>>,
}

impl<O: PullOp<Outflow = Df>> BlockingIntervalOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self { 
            op,
            interval,
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}

impl<O: PullOp<Outflow = Df>> Op for BlockingIntervalOp<O> {}

impl<O: PullOp<Outflow = Df>> PullOp for BlockingIntervalOp<O> {
    type Outflow = Df;
    type Outdomain<'s> = O::Outdomain<'s>;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
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
pub struct LeakyIntervalOp<O: PullOp<Outflow = Rx>> {
    op: O,
    interval: Duration,
    sleep: Pin<Box<Sleep>>,
}

impl<O: PullOp<Outflow = Rx>> LeakyIntervalOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op,
            interval,
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}

impl<O: PullOp<Outflow = Rx>> Op for LeakyIntervalOp<O> {}

impl<O: PullOp<Outflow = Rx>> PullOp for LeakyIntervalOp<O> {
    type Outflow = Rx;
    type Outdomain<'s> = O::Outdomain<'s>;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
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
pub struct BatchingOp<O, T>
where
    for<'a> O: PullOp<Outflow = Df, Outdomain<'a> = T>, 
{
    op: O,
    interval: Duration,
    buffer: VecDeque<T>,
    sleep: Pin<Box<Sleep>>,
}

impl<O, T> BatchingOp<O, T>
where
    for<'a> O: PullOp<Outflow = Df, Outdomain<'a> = T>, 
{
    pub fn new(op: O, interval: Duration) -> Self {
        Self {
            op,
            interval,
            buffer: VecDeque::new(),
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}

impl<O, T> Op for BatchingOp<O, T>
where
    for<'a> O: PullOp<Outflow = Df, Outdomain<'a> = T>, 
{}

impl<O, T> PullOp for BatchingOp<O, T>
where
    for<'a> O: PullOp<Outflow = Df, Outdomain<'a> = T>, 
{
    type Outflow = O::Outflow;
    type Outdomain<'s> = T;

    fn poll_next<'s>(&'s mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain<'s>>> {
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
