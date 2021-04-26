use std::task::{Context, Poll};

use super::*;

pub struct NullOp<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> NullOp<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'s, T: 's> Op<'s> for NullOp<T> {
    type Outdomain = T;

    fn poll_value(&'s self, _flow_type: FlowType, _ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        Poll::Pending
    }
}
