use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use crate::flow::*;

use super::*;

/// Create a connected sending and receiving handoff pair, [`HandoffPushOp`] and [`HandoffPullOp`].
///
/// This is meant as a lightweight single-threaded single-item-buffered alternative to [`channel_op`].
///
/// Supports both [`Df`] and [`Rx`] flows.
///
/// TODO: NOTE: [`Rx`] flows end up in the single element buffer, so they may unneccearily block.
pub fn handoff_op<F: Flow, T>() -> (HandoffPushOp<F, T>, HandoffPullOp<F, T>) {
    let handoff = Default::default();
    let handoff = Rc::new(RefCell::new(handoff));
    (
        HandoffPushOp::create(handoff.clone()),
        HandoffPullOp::create(handoff),
    )
}

/// Internal handoff struct.
struct Handoff<F: Flow, T> {
    item: Option<T>,
    recv_waker: Option<Waker>,
    send_waker: Option<Waker>,
    _phantom: std::marker::PhantomData<F>,
}
impl<F: Flow, T> Default for Handoff<F, T> {
    fn default() -> Self {
        Self {
            item: None,
            recv_waker: None,
            send_waker: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Internal future struct for sending half of handoff.
struct HandoffSend<F: Flow, T> {
    item: Option<T>,
    handoff: Rc<RefCell<Handoff<F, T>>>,
}
impl<F: Flow, T> Unpin for HandoffSend<F, T> {}
impl<F: Flow, T> Future for HandoffSend<F, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.item.take() {
            Some(item) => {
                let mut handoff_mut = this.handoff.borrow_mut();
                if handoff_mut.item.is_none() {
                    // Buffer has space, add.
                    handoff_mut.item.replace(item);
                    // Wake up receiver.
                    if let Some(waker) = &handoff_mut.recv_waker {
                        waker.wake_by_ref()
                    }
                    // Done.
                    Poll::Ready(())
                } else {
                    // Buffer full, wait.
                    let old_waker = handoff_mut.send_waker.replace(ctx.waker().clone());
                    assert!(old_waker.is_none()); // Does not allow multiple producer.
                    Poll::Pending
                }
            }
            // Already pushed.
            None => Poll::Ready(()),
        }
    }
}

/// The sending (push) half of a handoff.
pub struct HandoffPushOp<F: Flow, T> {
    handoff: Rc<RefCell<Handoff<F, T>>>,
}
impl<F: Flow, T> HandoffPushOp<F, T> {
    fn create(handoff: Rc<RefCell<Handoff<F, T>>>) -> Self {
        Self { handoff }
    }
}
impl<F: Flow, T> Op for HandoffPushOp<F, T> {}
impl<F: Flow, T> PushOp for HandoffPushOp<F, T> {
    type Inflow = F;
    type Indomain = T;
}
impl<F: Flow, T> MovePushOp for HandoffPushOp<F, T> {
    type Feedback = impl Future;

    #[must_use]
    fn push(&mut self, item: Self::Indomain) -> Self::Feedback {
        HandoffSend {
            item: Some(item),
            handoff: self.handoff.clone(),
        }
    }
}

/// The receiving (pull) half of a handoff.
pub struct HandoffPullOp<F: Flow, T> {
    handoff: Rc<RefCell<Handoff<F, T>>>,
}
impl<F: Flow, T> HandoffPullOp<F, T> {
    fn create(handoff: Rc<RefCell<Handoff<F, T>>>) -> Self {
        Self { handoff }
    }
}
impl<F: Flow, T> Op for HandoffPullOp<F, T> {}
impl<F: Flow, T> PullOp for HandoffPullOp<F, T> {
    type Outflow = F;
    type Outdomain = T;
}
impl<F: Flow, T> MovePullOp for HandoffPullOp<F, T> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        let mut handoff_mut = self.handoff.borrow_mut();
        match handoff_mut.item.take() {
            Some(item) => {
                handoff_mut.recv_waker.take();
                Poll::Ready(Some(item))
            }
            None => {
                handoff_mut.recv_waker.replace(ctx.waker().clone());
                Poll::Pending
            }
        }
    }
}
