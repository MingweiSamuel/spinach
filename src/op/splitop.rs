use std::borrow::ToOwned;
use std::cell::{RefCell,RefMut};
use std::task::{Context, Poll};

use super::*;

// struct SplitOpInternal<'p, O: 'static + Op> {
//     op: O,
//     val: O::Outdomain<'p>,
// }

pub struct SplitOpLead<'s, O: Op<'s>>
where
    O::Outdomain: Clone,
{
    op: O,
    split: &'s RefCell<Option<O::Outdomain>>,
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

        if self.split.borrow().is_none() {
            match self.op.poll_value(flow_type, ctx) {
                Poll::Ready(Some(delta)) => {
                    let old_opt = self.split.borrow_mut().replace(delta.clone());
                    assert!(old_opt.is_none());
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