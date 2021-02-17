use std::future::Future;
use std::task::{ Context, Poll };

use super::op::MovePullOp;


pub trait PureFn {
    type Indomain;
    type Outdomain;
    fn call(&self, item: Self::Indomain) -> Self::Outdomain;
}

pub trait PureRefFn {
    type Indomain;
    type Outdomain;
    fn call(&self, item: &Self::Indomain) -> Self::Outdomain;
}


pub struct MoveNext<'a, O: MovePullOp> {
    op: &'a mut O,
}
impl<'a, O: MovePullOp> MoveNext<'a, O> {
    pub fn new(op: &'a mut O) -> Self {
        Self {
            op: op,
        }
    }
}
impl<O: MovePullOp> Future for MoveNext<'_, O>
where
    Self: Unpin,
{
    type Output = Option<O::Outdomain>;

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
//     type Output = Option<&'b O::Outdomain>;

//     fn poll(self: std::pin::Pin<&'b mut RefNext<'a, O>>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
//         (*self).op.poll_next(ctx)
//         // (*x).op.poll_next(ctx)
//         // //op.poll_next(ctx)
//     }
// }

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
            }
            else {
                self.yielded = true;
                // cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    SleepYieldNow { yielded: false }.await
}
