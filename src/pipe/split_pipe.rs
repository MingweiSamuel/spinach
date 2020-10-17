use super::Pipe;

// TODO: may want to generalize this to more than two elements.
/// A pipe which allows you to split elements between two output pipes.
pub struct SplitPipe<L, R, F>
where
    L: Pipe,
    R: Pipe<Item = <L as Pipe>::Item>,
    F: Fn(&<L as Pipe>::Item) -> bool,
{
    pipe_l: L,
    pipe_r: R,
    splitter: F,
}

impl <L, R, F> SplitPipe<L, R, F>
where
    L: Pipe,
    R: Pipe<Item = <L as Pipe>::Item>,
    F: Fn(&<L as Pipe>::Item) -> bool,
{
    pub fn new(pipe_l: L, pipe_r: R, splitter: F) -> Self {
        Self {
            pipe_l: pipe_l,
            pipe_r: pipe_r,
            splitter: splitter,
        }
    }
}

impl <L, R, F> Pipe for SplitPipe<L, R, F>
where
    L: Pipe,
    R: Pipe<Item = Self::Item>,
    F: Fn(&<L as Pipe>::Item) -> bool,
{
    type Item = <L as Pipe>::Item;

    fn merge_in(&self, input: Self::Item) {
        if (self.splitter)(&input) {
            self.pipe_l.merge_in(input);
        }
        else {
            self.pipe_r.merge_in(input);
        }
    }
}
