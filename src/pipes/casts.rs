use super::*;

//
// SharedRefPipe <--- SharedMovePipe
//      ^                    ^
//      |                    |
//  ExclRefPipe  <---  ExclMovePipe
//

// SharedRefPipe <--- SharedMovePipe
pub struct SharedMoveFromSharedRefPipe<P: SharedRefOp>(P);
impl<P: SharedRefOp> Op for SharedMoveFromSharedRefPipe<P> {
    type Item = P::Item;
}
impl<P: SharedRefOp> SharedMoveOp for SharedMoveFromSharedRefPipe<P> {
    type Feedback = P::Feedback;

    fn push(&self, item: Self::Item) -> Self::Feedback {
        self.0.push(&item)
    }
}

// SharedRefPipe <--- ExclRefPipe
pub struct ExclRefFromSharedRefPipe<P: SharedRefOp>(P);
impl<P: SharedRefOp> Op for ExclRefFromSharedRefPipe<P> {
    type Item = P::Item;
}
impl<P: SharedRefOp> ExclRefOp for ExclRefFromSharedRefPipe<P> {
    type Feedback = P::Feedback;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        self.0.push(item)
    }
}

// ExclRefPipe <--- ExclMovePipe
pub struct ExclMovePipeFromExclRefPipe<P: ExclRefOp>(P);
impl<P: ExclRefOp> Op for ExclMovePipeFromExclRefPipe<P> {
    type Item = P::Item;
}
impl<P: ExclRefOp> ExclMoveOp for ExclMovePipeFromExclRefPipe<P> {
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Item) -> Self::Feedback {
        self.0.push(&item)
    }
}

// SharedMovePipe <--- ExclMovePipe
pub struct ExclMovePipeFromSharedMovePipe<P: SharedMoveOp>(P);
impl<P: SharedMoveOp> Op for ExclMovePipeFromSharedMovePipe<P> {
    type Item = P::Item;
}
impl<P: SharedMoveOp> ExclMoveOp for ExclMovePipeFromSharedMovePipe<P> {
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Item) -> Self::Feedback {
        self.0.push(item)
    }
}
