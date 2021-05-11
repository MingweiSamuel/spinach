use std::cell::{RefCell};
use std::rc::{Rc};
use std::task::{Context, Poll, Waker};

use crate::hide::{Hide, Delta, Value};

use super::*;


#[test]
pub fn test_construction() {
    use crate::lattice::ord::MaxRepr;

    let op = NullOp::<MaxRepr<String>>::new();
    let op = LatticeOp::<_, MaxRepr<String>>::new(op, "Hi".to_owned());

    let splitter = Splitter::new(op);

    let split0 = splitter.add_split();
    let split0 = LatticeOp::<_, MaxRepr<String>>::new(split0, "Wah".to_owned());

    let split1 = splitter.add_split();
    let split1 = LatticeOp::<_, MaxRepr<String>>::new(split1, "Hah".to_owned());

    let merge = MergeOp::new(split0, split1);
}

pub struct Splitter<O: OpValue> {
    op: O,
    splits: RefCell<Vec<SplitState<O>>>,
}
impl<O: OpValue> Splitter<O> {
    pub fn new(op: O) -> Self {
        Self {
            op,
            splits: Default::default(),
        }
    }

    pub fn add_split(&self) -> SplitOp<'_, O> {
        let mut splits = self.splits.borrow_mut();
        let index = splits.len();
        splits.push(SplitState::default());

        SplitOp {
            op: &self.op,
            splits: &self.splits,
            index,
        }
    }
}


pub struct SplitOp<'s, O: OpValue> {
    op: &'s O,
    splits: &'s RefCell<Vec<SplitState<O>>>,
    index: usize,
}

impl<'s, O: OpValue> Op for SplitOp<'s, O> {
    type LatRepr = O::LatRepr;
}

impl<'s, O: OpValue + OpDelta> OpDelta for SplitOp<'s, O> {
    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        let mut splits = self.splits.borrow_mut();

        // Check if other splits are ready to receive a value.
        for i in 0..splits.len() {
            let split = &mut splits[i];
            if self.index == i {
                split.waker.replace(ctx.waker().clone());
                continue;
            }
            if let Some(_) = split.delta {
                if let Some(waker) = &split.waker {
                    waker.wake_by_ref();
                }
                return Poll::Pending;
            }
        }

        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {
                for split in splits.iter_mut() {
                    let old_delta_opt = split.delta.replace(delta.clone());
                    assert!(old_delta_opt.is_none());

                    if let Some(waker) = split.waker.take() {
                        waker.wake();
                    }
                }
                Poll::Ready(Some(delta))
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'s, O: OpValue> OpValue for SplitOp<'s, O> {
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        self.op.get_value()
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
