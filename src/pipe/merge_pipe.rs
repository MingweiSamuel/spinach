use std::rc::Rc;

use super::Pipe;

/// A pipe which can be cloned to merge multiple pipes together.
#[derive(Clone)]
pub struct MergePipe<P: Pipe> {
    pipe: Rc<P>,
}
impl <P: Pipe> MergePipe<P> {
    pub fn new(pipe: P) -> Self {
        Self {
            pipe: Rc::new(pipe),
        }
    }
}
impl <P: Pipe> Pipe for MergePipe<P> {
    type Item = <P as Pipe>::Item;

    fn merge_in(&self, input: Self::Item) {
        self.pipe.merge_in(input);
    }
}
