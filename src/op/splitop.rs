use std::cell::{RefCell};
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use super::*;

pub fn split_follow<'s, O: Op<'s>>() -> SplitOpFollow<'s, O>
where
    O::Outdomain: Clone,
{
    SplitOpFollow::<'s, O> {
        split: Rc::new(RefCell::default()),
    }
}

pub fn split_lead<'s, O: Op<'s>>(op: O, follow: &SplitOpFollow<'s, O>) -> SplitOpLead<'s, O>
where
    O::Outdomain: Clone,
{
    SplitOpLead::<'s, O> {
        op,
        split: follow.split.clone(),
    }
}

#[test]
pub fn test_construction() {
    use crate::lattice::Max;

    let op0 = NullOp::<String>::new();
    let op1 = LatticeOp::<_, Max<String>>::new(op0, "Hi".to_owned());

    let follow = split_follow();
    let op2 = split_lead(op1, &follow);

    let _ = op2;
}


pub struct SplitOpLead<'s, O: Op<'s>>
where
    O::Outdomain: Clone,
{
    op: O,
    split: Rc<RefCell<(Option<Waker>, Option<O::Outdomain>)>>,
}

impl<'s, O: Op<'s>> Op<'s> for SplitOpLead<'s, O>
where
    O::Outdomain: Clone,
{
    type Outdomain = O::Outdomain;
}

impl<'s, O: OpDelta<'s>> OpDelta<'s> for SplitOpLead<'s, O>
where
    O::Outdomain: Clone,
{
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        let mut borrow = self.split.borrow_mut();
        if borrow.1.is_none() {
            match self.op.poll_delta(ctx) {
                Poll::Ready(Some(delta)) => {
                    let old_opt = borrow.1.replace(delta.clone());
                    assert!(old_opt.is_none());
                    if let Some(waker) = borrow.0.take() {
                        waker.wake();
                    }
                    Poll::Ready(Some(delta))
                },
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
        else {
            Poll::Pending
        }
    }
}

pub struct SplitOpFollow<'s, O: Op<'s>>
where
    O::Outdomain: Clone,
{
    split: Rc<RefCell<(Option<Waker>, Option<O::Outdomain>)>>,
}

impl<'s, O: Op<'s>> Op<'s> for SplitOpFollow<'s, O>
where
    O::Outdomain: Clone,
{
    type Outdomain = O::Outdomain;
}

impl<'s, O: OpDelta<'s>> OpDelta<'s> for SplitOpFollow<'s, O>
where
    O::Outdomain: Clone,
{
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        let mut borrow = self.split.borrow_mut();
        borrow.0.replace(ctx.waker().clone());
        match borrow.1.take() {
            Some(val) => Poll::Ready(Some(val)),
            None => Poll::Pending,
        }
    }
}
