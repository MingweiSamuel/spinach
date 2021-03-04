use std::convert::AsRef;

use super::*;

/// An Op for converting an owned flow into a reference flow via [`AsRef`].
///
/// Note: Only supports Push. Supports both [`Df`] and [`Rx`].
///
/// To go from ref to owned (the opposite of this), use [`ToOwnedOp`].
pub struct AsRefOp<O: Op, T: AsRef<U>, U> {
    op: O,
    _phantom: std::marker::PhantomData<(T, U)>,
}

impl<O: Op, T: AsRef<U>, U> AsRefOp<O, T, U> {
    pub fn new(op: O) -> Self {
        Self { op, _phantom: std::marker::PhantomData }
    }
}

impl<O: Op, T: AsRef<U>, U> Op for AsRefOp<O, T, U> {}

impl<O: Op, T: AsRef<U>, U> PushOp for AsRefOp<O, T, U>
where
    for<'a> O: PushOp<Indomain<'a> = &'a U>,
{
    type Inflow = O::Inflow;
    type Indomain<'p> = T;

    type Feedback = O::Feedback;

    fn push<'p>(&mut self, item: Self::Indomain<'p>) -> Self::Feedback {
        self.op.push(item.as_ref())
    }
}
