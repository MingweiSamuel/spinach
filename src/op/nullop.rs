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
}

impl<'s, T: 's> OpDelta<'s> for NullOp<T> {
    fn poll_delta(&'s self, _ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        Poll::Pending
    }
}

impl<'s, T: 's> OpValue<'s> for NullOp<T> {
    fn poll_value(&'s self, _ctx: &mut Context<'_>) -> Poll<Self::Outdomain> {
        Poll::Pending
    }
}
