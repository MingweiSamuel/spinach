use std::cell::{Cell, RefCell};
use std::task::{Context, Poll, Waker};
use std::rc::{Rc, Weak};

use crate::hide::{Hide, Delta, Value};

use super::*;


pub fn fixed_split<O: Op, const N: usize>(op: O) -> [SplitOp<O>; N] {
    let splitter = Splitter::new(op);
    [(); N].map(|_| splitter.internal_add_split_reveal())
}


struct SplitterState<O: Op> {
    op: O,
    closed: Cell<bool>,
    splits: RefCell<Vec<Weak<RefCell<SplitState<O>>>>>,
}

pub struct Splitter<O: Op> {
    state: Rc<SplitterState<O>>,
}

impl<O: Op> Splitter<O> {
    pub fn new(op: O) -> Self {
        let state = Rc::new(SplitterState {
            op,
            closed: Cell::new(false),
            splits: Default::default(),
        });
        Self { state }
    }

    #[must_use]
    fn internal_add_split_reveal(&self) -> SplitOp<O> {
        let mut splits = self.state.splits.borrow_mut();
        let split = Rc::new(RefCell::default());
        splits.push(Rc::downgrade(&split));

        SplitOp {
            splitter: self.state.clone(),
            split,
        }
    }
}

impl<O: OpValue> Splitter<O> {
    #[must_use]
    pub fn add_split(&self) -> SplitOp<O> {
        self.internal_add_split_reveal()
    }
}

impl<O: Op> Clone for Splitter<O> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}




pub struct SplitOp<O: Op> {
    splitter: Rc<SplitterState<O>>,
    split: Rc<RefCell<SplitState<O>>>,
}

impl<O: Op> Op for SplitOp<O> {
    type LatRepr = O::LatRepr;
}

impl<O: OpDelta> OpDelta for SplitOp<O> {
    type Ord = O::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        if self.splitter.closed.get() {
            return Poll::Ready(None);
        }

        // Check if we have a value waiting.
        {
            let mut split = self.split.borrow_mut();
            match split.delta.take() {
                Some(polled) => {
                    return Poll::Ready(Some(polled));
                }
                None => {
                    split.waker.replace(ctx.waker().clone());
                }
            }
        }

        // Remove any weak (removed) splits.
        let mut splits = Vec::new();
        {
            self.splitter.splits.borrow_mut().retain(|split_weak| {
                match split_weak.upgrade() {
                    Some(split) => {
                        splits.push(split);
                        true
                    }
                    None => false
                }
            });
        }
        // Get our index.
        let index = splits.iter().enumerate().find_map(|(i, split)| {
            if Rc::ptr_eq(&self.split, split) {
                Some(i)
            }
            else {
                None
            }
        }).expect("WE DONT EXIST :C");

        // Iterate in circular order, so each successive split checks the next split.
        let (splits_before, splits_after) = splits.split_at_mut(index);
        let splits_after = &mut splits_after[1..]; // Skip self.

        // Check if other splits are ready to receive a value.
        for split in splits_after.iter().chain(splits_before.iter()) {
            let split = split.borrow();
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
                    let mut split = split.borrow_mut();
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

impl<O: OpValue> OpValue for SplitOp<O> {
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        self.splitter.op.get_value()
    }
}



struct SplitState<O: Op> {
    waker: Option<Waker>,
    delta: Option<Hide<Delta, O::LatRepr>>,
}

impl<O: Op> Default for SplitState<O> {
    fn default() -> Self {
        Self {
            waker: None,
            delta: None,
        }
    }
}
