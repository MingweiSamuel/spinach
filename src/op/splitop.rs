use std::cell::{RefCell};
use std::rc::{Rc};
use std::task::{Context, Poll, Waker};

use super::*;


// #[test]
// pub fn test_construction() {
//     use crate::lattice::Max;

//     let op0 = NullRefOp::<String>::new();
//     let op0 = CloneOp::<_>::new(op0);
//     let op1 = LatticeOp::<_, Max<String>>::new(op0, "Hi".to_owned());
//     let op2 = SplitOp::new(op1);
    
//     let follow0 = op2.get_split();
//     let _ = follow0;
//     // let follow1 = LatticeOp::<_, Max<String>>::new(follow0, "No".to_owned());

//     let _ = op2;
// }


pub struct SplitOp<'s, O> {
    op: O,
    split: Rc<RefCell<FollowState<O::Outdomain>>>,
}

impl<'s, O: OpValue<'s>> SplitOp<'s, O>
where
    O::Outdomain: Clone,
{
    pub fn new(op: O) -> Self {
        Self {
            op,
            split: Rc::default(),
        }
    }

    pub fn get_split(&'s self) -> SplitOpFollow<'s, O> {
        SplitOpFollow {
            value: self.get_value(),
            split: self.split.clone(),
        }
    }
}

impl<'s, O: Op<'s>> Op<'s> for SplitOp<'s, O>
where
    O::Outdomain: Clone,
{
    type Outdomain = O::Outdomain;
}

impl<'s, O: OpDelta<'s>> OpDelta<'s> for SplitOp<'s, O>
where
    O::Outdomain: Clone,
{
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
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

impl<'s, O: OpValue<'s>> OpValue<'s> for SplitOp<'s, O>
where
    O::Outdomain: Clone,
{
    fn get_value(&'s self) -> Self::Outdomain {
        self.op.get_value()
    }
}



struct FollowState<T> {
    lead_waker: Option<Waker>,
    follow_waker: Option<Waker>,
    delta: Option<T>,
}

impl<T> Default for FollowState<T> {
    fn default() -> Self {
        Self {
            lead_waker: None,
            follow_waker: None,
            delta: None,
        }
    }
}

pub struct SplitOpFollow<'s, O: Op<'s>> {
    value: O::Outdomain,
    split: Rc<RefCell<FollowState<O::Outdomain>>>,
}

impl<'s, O: Op<'s>> Op<'s> for SplitOpFollow<'s, O> {
    type Outdomain = O::Outdomain;
}

impl<'s, O: OpDelta<'s>> OpDelta<'s> for SplitOpFollow<'s, O> {
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
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

impl<'s, O: OpValue<'s>> OpValue<'s> for SplitOpFollow<'s, O>
where
    O::Outdomain: Clone,
{
    fn get_value(&'s self) -> Self::Outdomain {
        self.value.clone()
    }
}
