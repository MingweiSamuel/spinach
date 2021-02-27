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
        ReferenceOp { op }
    }
}
impl<O: Op> Op for ReferenceOp<O> {}

impl<O: PushOp> PushOp for ReferenceOp<O> {
    type Inflow = O::Inflow;
    type Indomain = O::Indomain;
}

impl<O: RefPushOp> MovePushOp for ReferenceOp<O>
where
    O::Indomain: Clone,
{
    type Feedback = O::Feedback;

    fn push(&mut self, item: Self::Indomain) -> Self::Feedback {
        self.op.push(&item)
    }
}
