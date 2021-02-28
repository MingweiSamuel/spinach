use super::*;

/// An Op for converting an owned flow into a reference flow.
/// Simply takes the reference of the owned value.
///
/// Note: Only supports Push. Supports both [`Df`] and [`Rx`].
///
/// To go from ref to owned (the opposite of this), use [`CloneOp`].
pub struct ReferenceOp<O: Op> {
    op: O,
}

impl<O: Op> ReferenceOp<O> {
    pub fn new(op: O) -> Self {
        Self { op }
    }
}

impl<O: Op> Op for ReferenceOp<O> {}

impl<T, O> PushOp for ReferenceOp<O>
where
    for<'a> O: PushOp<Indomain<'a> = &'a T>,
{
    type Inflow = O::Inflow;
    type Indomain<'p> = T;

    type Feedback = O::Feedback;

    fn push<'p>(&mut self, item: Self::Indomain<'p>) -> Self::Feedback {
        self.op.push(&item)
    }
}
