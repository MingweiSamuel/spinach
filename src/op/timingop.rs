use std::future::Future;
use std::pin::Pin;
use std::task::{ Context, Poll };
use std::time::Duration;

use tokio::time::{ self, Sleep };

use super::op::*;
use super::flow::Flow;


pub struct RateLimitOp<O: Op> {
    op: O,
    interval: Duration,
    sleep: Pin<Box<Sleep>>,
}
impl<O: Op> RateLimitOp<O> {
    pub fn new(op: O, interval: Duration) -> Self {
        RateLimitOp {
            op: op,
            interval: interval,
            sleep: Box::pin(time::sleep(interval)),
        }
    }
}
impl<O: Op> Op for RateLimitOp<O> {}
impl<O: PullOp> PullOp for RateLimitOp<O> {
    type Outflow = O::Outflow;
}
impl<O: PushOp> PushOp for RateLimitOp<O> {
    type Inflow = O::Inflow;
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
// impl<O: MovePushOp> MovePushOp for RateLimitOp<O> {
//     type Feedback = impl Future;

//     fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
//         async move {
//             let item = item;
//             self.op.push(item).await;
//         }
//         // println!("{}: {:?}", self.tag, item);
//         // self.op.push(item)
//     }
// }
// impl<O: RefPushOp> RefPushOp for RateLimitOp<O> {
//     type Feedback = O::Feedback;

//     fn push(&mut self, item: &<Self::Inflow as Flow>::Domain) -> Self::Feedback {
//         println!("{}: {:?}", self.tag, item);
//         self.op.push(item)
//     }
// }
