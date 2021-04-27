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

pub struct NullRefOp<T: ?Sized> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ?Sized> NullRefOp<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'s, T: 's + ?Sized> Op<'s> for NullRefOp<T> {
    type Outdomain = &'s T;
}

impl<'s, T: 's + ?Sized> OpDelta<'s> for NullRefOp<T> {
    fn poll_delta(&'s self, _ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        Poll::Pending
    }
}
