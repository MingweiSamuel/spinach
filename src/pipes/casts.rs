use super::*;

//
// SharedRefPipe <--- SharedMovePipe
//      ^                    ^
//      |                    |
//  ExclRefPipe  <---  ExclMovePipe
//

// SharedRefPipe <--- SharedMovePipe
pub struct SharedMoveFromSharedRefPipe<P: SharedRefPipe>(P);
impl<P: SharedRefPipe> Pipe for SharedMoveFromSharedRefPipe<P> {
    type Item = P::Item;
}
impl<P: SharedRefPipe> SharedMovePipe for SharedMoveFromSharedRefPipe<P> {
    type Feedback = P::Feedback;

    fn push(&self, item: Self::Item) -> Self::Feedback {
        self.0.push(&item)
    }
}

// SharedRefPipe <--- ExclRefPipe
pub struct ExclRefFromSharedRefPipe<P: SharedRefPipe>(P);
impl<P: SharedRefPipe> Pipe for ExclRefFromSharedRefPipe<P> {
    type Item = P::Item;
}
impl<P: SharedRefPipe> ExclRefPipe for ExclRefFromSharedRefPipe<P> {
    type Feedback = P::Feedback;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        self.0.push(item)
    }
}

// ExclRefPipe <--- ExclMovePipe
pub struct ExclMovePipeFromExclRefPipe<P: ExclRefPipe>(P);
impl<P: ExclRefPipe> Pipe for ExclMovePipeFromExclRefPipe<P> {
    type Item = P::Item;
}
impl<P: ExclRefPipe> ExclMovePipe for ExclMovePipeFromExclRefPipe<P> {
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Item) -> Self::Feedback {
        self.0.push(&item)
    }
}

// SharedMovePipe <--- ExclMovePipe
pub struct ExclMovePipeFromSharedMovePipe<P: SharedMovePipe>(P);
impl<P: SharedMovePipe> Pipe for ExclMovePipeFromSharedMovePipe<P> {
    type Item = P::Item;
}
impl<P: SharedMovePipe> ExclMovePipe for ExclMovePipeFromSharedMovePipe<P> {
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Item) -> Self::Feedback {
        self.0.push(item)
    }
}
