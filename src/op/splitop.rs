use std::cell::{RefCell};
use std::task::{Context, Poll, Waker};

use super::*;

pub fn split_follow<'s, O: Op<'s>>() -> SplitOpFollow<'s, O>
where
    O::Outdomain: Clone,
{
    SplitOpFollow::<'s, O> {
        split: RefCell::default(),
    }
}

pub fn split_lead<'s, O: Op<'s>>(op: O, follow: &'s SplitOpFollow<'s, O>) -> SplitOpLead<'s, O>
where
    O::Outdomain: Clone,
{
    SplitOpLead::<'s, O> {
        op,
        split: &follow.split,
    }
}

#[test]
pub fn test_construction() {
    use crate::lattice::Max;

    let op0 = NullOp::<String>::new();
    let op1 = LatticeOp::<_, Max<String>>::new(op0, "Hi".to_owned());

    let follow = split_follow();
    let op2 = split_lead(op1, &follow);

    std::mem::drop(op2);
}


pub struct SplitOpLead<'s, O: Op<'s>>
where
    O::Outdomain: Clone,
{
    op: O,
    split: &'s RefCell<(Option<Waker>, Option<O::Outdomain>)>,
}

impl<'s, O: Op<'s>> Op<'s> for SplitOpLead<'s, O>
where
    O::Outdomain: Clone,
{
    type Outdomain = O::Outdomain;

    fn poll_value(&'s self, flow_type: FlowType, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        if let FlowType::Value = flow_type {
            todo!();
        }

        let mut borrow = self.split.borrow_mut();
        if borrow.1.is_none() {
            match self.op.poll_value(flow_type, ctx) {
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
    split: RefCell<(Option<Waker>, Option<O::Outdomain>)>,
}

impl<'s, O: Op<'s>> Op<'s> for SplitOpFollow<'s, O>
where
    O::Outdomain: Clone,
{
    type Outdomain = O::Outdomain;

    fn poll_value(&'s self, flow_type: FlowType, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        if let FlowType::Value = flow_type {
            todo!();
        }

        let mut borrow = self.split.borrow_mut();
        borrow.0.replace(ctx.waker().clone());
        match borrow.1.take() {
            Some(val) => Poll::Ready(Some(val)),
            None => Poll::Pending,
        }
    }
}
