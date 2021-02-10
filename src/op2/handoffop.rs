use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{ Context, Poll, Waker };

use super::op::*;



pub fn handoff_op<T>() -> ( HandoffPushOp<T>, HandoffPullOp<T> ) {
    let handoff = Default::default();
    let handoff = Rc::new(RefCell::new(handoff));
    ( HandoffPushOp::create(handoff.clone()), HandoffPullOp::create(handoff) )
}

struct Handoff<T> {
    item: Option<T>,
    recv_waker: Option<Waker>,
    send_waker: Option<Waker>,
}
impl<T> Default for Handoff<T> {
    fn default() -> Self {
        Self {
            item: None,
            recv_waker: None,
            send_waker: None,
        }
    }
} 


struct HandoffSend<T> {
    item: Option<T>,
    handoff: Rc<RefCell<Handoff<T>>>,
}
impl<T> Unpin for HandoffSend<T> {}
impl<T> Future for HandoffSend<T> {
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
                }
                else {
                    // Buffer full, wait.
                    let old_waker = handoff_mut.send_waker.replace(ctx.waker().clone());
                    assert!(old_waker.is_none()); // Does not allow multiple producer.
                    Poll::Pending
                }
            },
            // Already pushed.
            None => Poll::Ready(()),
        }
    }
}



pub struct HandoffPushOp<T> {
    handoff: Rc<RefCell<Handoff<T>>>,
}
impl<T> HandoffPushOp<T> {
    fn create(handoff: Rc<RefCell<Handoff<T>>>) -> Self {
        Self {
            handoff: handoff,
        }
    }
}
impl<T> Op for HandoffPushOp<T> {}
impl<T> PushOp for HandoffPushOp<T> {
    type Domain = T;
}
impl<T> MovePushOp for HandoffPushOp<T> {
    type Feedback = impl Future;

    #[must_use]
    fn push(&mut self, item: Self::Domain) -> Self::Feedback {
        HandoffSend {
            item: Some(item),
            handoff: self.handoff.clone(),
        }
    }
}



pub struct HandoffPullOp<T> {
    handoff: Rc<RefCell<Handoff<T>>>,
}
impl<T> HandoffPullOp<T> {
    fn create(handoff: Rc<RefCell<Handoff<T>>>) -> Self {
        Self {
            handoff: handoff,
        }
    }
}
impl<T> Op for HandoffPullOp<T> {}
impl<T> PullOp for HandoffPullOp<T> {
    type Codomain = T;
}
impl<T> MovePullOp for HandoffPullOp<T> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>> {
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
