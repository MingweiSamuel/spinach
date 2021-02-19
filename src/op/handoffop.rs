use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{ Context, Poll, Waker };

use super::op::*;
use super::flow::*;



pub fn handoff_op<F: Flow>() -> ( HandoffPushOp<F>, HandoffPullOp<F> ) {
    let handoff = Default::default();
    let handoff = Rc::new(RefCell::new(handoff));
    ( HandoffPushOp::create(handoff.clone()), HandoffPullOp::create(handoff) )
}

struct Handoff<F: Flow> {
    item: Option<F::Domain>,
    recv_waker: Option<Waker>,
    send_waker: Option<Waker>,
    _phantom: std::marker::PhantomData<F>,
}
impl<F: Flow> Default for Handoff<F> {
    fn default() -> Self {
        Self {
            item: None,
            recv_waker: None,
            send_waker: None,
            _phantom: std::marker::PhantomData,
        }
    }
} 


struct HandoffSend<F: Flow> {
    item: Option<F::Domain>,
    handoff: Rc<RefCell<Handoff<F>>>,
}
impl<F: Flow> Unpin for HandoffSend<F> {}
impl<F: Flow> Future for HandoffSend<F> {
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



pub struct HandoffPushOp<F: Flow> {
    handoff: Rc<RefCell<Handoff<F>>>,
}
impl<F: Flow> HandoffPushOp<F> {
    fn create(handoff: Rc<RefCell<Handoff<F>>>) -> Self {
        Self {
            handoff: handoff,
        }
    }
}
impl<F: Flow> Op for HandoffPushOp<F> {}
impl<F: Flow> PushOp for HandoffPushOp<F> {
    type Inflow = F;
}
impl<F: Flow> MovePushOp for HandoffPushOp<F> {
    type Feedback = impl Future;

    #[must_use]
    fn push(&mut self, item: <Self::Inflow as Flow>::Domain) -> Self::Feedback {
        HandoffSend {
            item: Some(item),
            handoff: self.handoff.clone(),
        }
    }
}



pub struct HandoffPullOp<F: Flow> {
    handoff: Rc<RefCell<Handoff<F>>>,
}
impl<F: Flow> HandoffPullOp<F> {
    fn create(handoff: Rc<RefCell<Handoff<F>>>) -> Self {
        Self {
            handoff: handoff,
        }
    }
}
impl<F: Flow> Op for HandoffPullOp<F> {}
impl<F: Flow> PullOp for HandoffPullOp<F> {
    type Outflow = F;
}
impl<F: Flow> MovePullOp for HandoffPullOp<F> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<<Self::Outflow as Flow>::Domain>> {
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
