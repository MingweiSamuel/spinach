use std::future::Future;
use std::task::{Context, Poll};

use super::{Flow, MovePullOp};

/// Helper future struct for getting a value from [`MovePullOp`]s.
pub struct MoveNext<'a, O: MovePullOp> {
    op: &'a mut O,
}
impl<'a, O: MovePullOp> MoveNext<'a, O> {
    pub fn new(op: &'a mut O) -> Self {
        Self { op }
    }
}
impl<O: MovePullOp> Future for MoveNext<'_, O>
where
    Self: Unpin,
{
    type Output = Option<<O::Outflow as Flow>::Domain>;

    fn poll(self: std::pin::Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().op.poll_next(ctx)
    }
}

// pub struct RefNext<'a, O: RefPullOp> {
//     op: &'a mut O,
// }
// impl<'a, O: RefPullOp> RefNext<'a, O> {
//     pub fn new(op: &'a mut O) -> Self {
//         Self {
//             op: op,
//         }
//     }
// }
// impl<'a, 'b, O: RefPullOp> Future for &'b mut RefNext<'a, O>
// where
//     Self: Unpin,
// {
//     type Output = Option<&'b <O::Outflow as Flow>::Domain>;

//     fn poll(self: std::pin::Pin<&'b mut RefNext<'a, O>>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
//         (*self).op.poll_next(ctx)
//         // (*x).op.poll_next(ctx)
//         // //op.poll_next(ctx)
//     }
// }

/// An async function which puts the current task to sleep.
/// Unlike [`tokio::task::yield_now`], this marks the current task as not ready, so it
/// will remain asleep until the task is awoken by an event.
pub async fn sleep_yield_now() {
    /// Yield implementation
    struct SleepYieldNow {
        yielded: bool,
    }

    impl Future for SleepYieldNow {
        type Output = ();

        fn poll(mut self: std::pin::Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<()> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                // cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    SleepYieldNow { yielded: false }.await
}
