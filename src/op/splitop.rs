use std::cell::{Cell, RefCell};
use std::task::{Context, Poll, Waker};

use crate::hide::{Hide, Delta, Value};

use super::*;

pub struct Splitter<O: OpValue> {
    op: O,
    closed: Cell<bool>,
    splits: RefCell<Vec<SplitState<O>>>,
}
impl<O: OpValue> Splitter<O> {
    pub fn new(op: O) -> Self {
        Self {
            op,
            closed: Cell::new(false),
            splits: Default::default(),
        }
    }

    #[must_use]
    pub fn add_split(&self) -> SplitOp<'_, O> {
        let mut splits = self.splits.borrow_mut();
        let index = splits.len();
        splits.push(SplitState::default());

        SplitOp {
            splitter: &self,
            index,
        }
    }
}


pub struct SplitOp<'s, O: OpValue> {
    splitter: &'s Splitter<O>,
    index: usize,
}

impl<'s, O: OpValue> Op for SplitOp<'s, O> {
    type LatRepr = O::LatRepr;
}

impl<'s, O: OpValue + OpDelta> OpDelta for SplitOp<'s, O> {
    type Ord = O::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        if self.splitter.closed.get() {
            return Poll::Ready(None);
        }

        let mut splits = self.splitter.splits.borrow_mut();

        // Check if we have a value waiting.
        {
            let split = &mut splits[self.index];
            match split.delta.take() {
                Some(polled) => {
                    return Poll::Ready(Some(polled));
                }
                None => {
                    split.waker.replace(ctx.waker().clone());
                }
            }
        }

        // Iterate in circular order, so each successive split checks the next split.
        let (splits_before, splits_after) = splits.split_at_mut(self.index);
        let splits_after = &mut splits_after[1..]; // Skip self.

        // Check if other splits are ready to receive a value.
        for split in splits_after.iter().chain(splits_before.iter()) {
            if let Some(_) = split.delta {
                // If any split has it's value filled, wake it up and return pending.
                if let Some(waker) = &split.waker {
                    waker.wake_by_ref();
                }
                return Poll::Pending;
            }
        }

        match self.splitter.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {
                for split in splits_after.iter_mut().chain(splits_before.iter_mut()) {
                    let old_delta_opt = split.delta.replace(delta.clone());
                    assert!(old_delta_opt.is_none());

                    if let Some(waker) = split.waker.take() {
                        waker.wake();
                    }
                }
                Poll::Ready(Some(delta))
            },
            Poll::Ready(None) => {
                self.splitter.closed.replace(true);
                Poll::Ready(None)
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'s, O: OpValue> OpValue for SplitOp<'s, O> {
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        self.splitter.op.get_value()
    }
}



struct SplitState<O: OpValue> {
    waker: Option<Waker>,
    delta: Option<Hide<Delta, O::LatRepr>>,
}

impl<O: OpValue> Default for SplitState<O> {
    fn default() -> Self {
        Self {
            waker: None,
            delta: None,
        }
    }
}
