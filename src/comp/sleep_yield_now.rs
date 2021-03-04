use std::future::Future;
use std::task::{Context, Poll};

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
