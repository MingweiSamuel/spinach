use std::task::{Context, Poll};

use super::*;


pub struct CloneOp<'s, O: Op<'s>>
where
    O::Outdomain: Clone,
{
    op: O,
    _phantom: &'s (),
}

impl<'s, O: Op<'s>> CloneOp<'s, O>
where
    O::Outdomain: Clone,
{
    pub fn new(op: O) -> Self {
        Self {
            op,
            _phantom: &(),
        }
    }
}

impl<'s, O, T: 's> Op<'s> for CloneOp<'s, O>
where
    O: Op<'s, Outdomain = &'s T>,
    T: Clone,
{
    type Outdomain = T;
}

impl<'s, O, T: 's> OpDelta<'s> for CloneOp<'s, O>
where
    O: OpDelta<'s, Outdomain = &'s T>,
    T: Clone,
{
    fn poll_delta(&'s self, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        self.op.poll_delta(ctx)
            .map(|opt| opt.map(|val| val.clone()))
    }
}

impl<'s, O, T: 's> OpValue<'s> for CloneOp<'s, O>
where
    O: OpValue<'s, Outdomain = &'s T>,
    T: Clone,
{
    fn get_value(&'s self) -> Self::Outdomain {
        self.op.get_value().clone()
    }
}
