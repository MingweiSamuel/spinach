use std::cell::{RefCell};
use std::rc::{Rc};
use std::task::{Context, Poll, Waker};

use crate::hide::{Hide, Delta, Value};

use super::*;


#[test]
pub fn test_construction() {
    use crate::lattice::ord::MaxRepr;

    let op = NullOp::<MaxRepr<String>>::new();
    let op = LatticeOp::<_, MaxRepr<String>>::new(op, "Hi".to_owned());
    
    let lead = SplitOp::new(&op);

    let follow = SplitOpFollow::new(&lead);
    let follow = LatticeOp::<_, MaxRepr<String>>::new(follow, "Hah".to_owned());

    let lead = LatticeOp::<_, MaxRepr<String>>::new(lead, "Wah".to_owned());

    let merge = MergeOp::new(lead, follow);
}


pub struct SplitOp<'s, O: OpValue> {
    op: &'s O,
    split: Rc<RefCell<FollowState<O>>>,
}

impl<'s, O: OpValue> SplitOp<'s, O> {
    pub fn new(op: &'s O) -> Self {
        Self {
            op,
            split: Default::default(),
        }
    }

    // pub fn get_split(&self) -> SplitOpFollow<'s, O> {
    //     let rcell = RefCell::default();
    //     self.split.replace(&rcell)
    //     SplitOpFollow {
    //         op: self.op,
    //         split: rcell,
    //     }
    // }
}

impl<'s, O: OpValue> Op for SplitOp<'s, O> {
    type LatRepr = O::LatRepr;
}

impl<'s, O: OpValue + OpDelta> OpDelta for SplitOp<'s, O> {
    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        let mut borrow = self.split.borrow_mut();
        match borrow.delta {
            None => {
                match self.op.poll_delta(ctx) {
                    Poll::Ready(Some(delta)) => {
                        let old_opt = borrow.delta.replace(delta.clone());
                        assert!(old_opt.is_none());
    
                        if let Some(waker) = borrow.follow_waker.take() {
                            waker.wake();
                        }
                        Poll::Ready(Some(delta))
                    },
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
            Some(_) => {
                borrow.lead_waker.replace(ctx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<'s, O: OpValue> OpValue for SplitOp<'s, O> {
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        self.op.get_value()
    }
}



struct FollowState<O: OpValue> {
    lead_waker: Option<Waker>,
    follow_waker: Option<Waker>,
    delta: Option<Hide<Delta, O::LatRepr>>,
}

impl<O: OpValue> Default for FollowState<O> {
    fn default() -> Self {
        Self {
            lead_waker: None,
            follow_waker: None,
            delta: None,
        }
    }
}



pub struct SplitOpFollow<'s, O: OpValue> {
    op: &'s O,
    split: Rc<RefCell<FollowState<O>>>,
}

impl<'s, O: OpValue> SplitOpFollow<'s, O> {
    pub fn new(lead: &SplitOp<'s, O>) -> Self {
        Self {
            op: lead.op,
            split: lead.split.clone(),
        }
    }
}

impl<'s, O: OpValue> Op for SplitOpFollow<'s, O> {
    type LatRepr = O::LatRepr;
}

impl<'s, O: OpValue + OpDelta> OpDelta for SplitOpFollow<'s, O> {
    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        let mut borrow = self.split.borrow_mut();
        match borrow.delta.take() {
            Some(val) => {
                if let Some(lead_waker) = borrow.lead_waker.take() {
                    lead_waker.wake();
                }
                Poll::Ready(Some(val))
            }
            None => {
                borrow.follow_waker.replace(ctx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<'s, O: OpValue> OpValue for SplitOpFollow<'s, O> {
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        self.op.get_value()
    }
}
