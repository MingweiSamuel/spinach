use std::cell::RefCell;
use std::task::{Context, Poll};

use crate::lattice::{Lattice};

use super::*;

pub enum LatticeWrapper<'s, O, F: Lattice>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    Delta {
        target: &'s LatticeOp<'s, O, F>,
        delta: Option<F::Domain>,
    },
    Value(&'s LatticeOp<'s, O, F>),
}

// impl<'s, O: 'static, F: 'static + Lattice> LatticeWrapper<'s, O, F>
// where
//     for<'a> O: Op<Outdomain<'a> = F::Domain>,
// {
//     pub fn hide(&'s self) -> Option<&'s Hide<F>> {
//         match self {
//             Self::Delta { target: _, delta } => {
//                 match delta {
//                     Some(delta) => Some(Hide::from_ref(delta)),
//                     None => None,
//                 }
//             }
//             Self::Value(target) => Some(Hide::from_ref(&*target.state.borrow())),
//         }
//     }
// }

impl<'s, O, F: Lattice> Drop for LatticeWrapper<'s, O, F>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    fn drop(&mut self) {
        if let Self::Delta { target, delta } = self {
            if let Some(delta) = delta.take() {
                F::merge_in(&mut target.state.borrow_mut(), delta);
            }
        }
    }
}

/// A state-accumulating lattice op.
///
/// Input is owned `F::Domain` values as [`Df`] dataflow,
/// output is reference `&F::Domain` values as [`Rx`] reactive.
pub struct LatticeOp<'s, O: 's, F: 's + Lattice>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    op: O,
    state: RefCell<F::Domain>,
    _phantom: &'s (),
}

impl<'s, O, F: Lattice> Op<'s> for LatticeOp<'s, O, F>
where
    O: Op<'s, Outdomain = F::Domain>,
{
    type Outdomain = LatticeWrapper<'s, O, F>;

    fn poll_value(&'s self, flow_type: FlowType, ctx: &mut Context<'_>) -> Poll<Option<Self::Outdomain>> {
        match flow_type {
            FlowType::Delta => {
                match self.op.poll_value(FlowType::Delta, ctx) {
                    Poll::Ready(Some(delta)) => Poll::Ready(Some(LatticeWrapper::Delta {
                        target: self,
                        delta: Some(delta),
                    })),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
            FlowType::Value => Poll::Ready(Some(LatticeWrapper::Value(&self)))
        }
    }
}
