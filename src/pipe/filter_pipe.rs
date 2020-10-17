use super::{ PipeConstructor, Pipe };

/// A pipe which maps elements.
pub struct FilterPipe<F, P>
where
    F: Fn(&<P as Pipe>::Item) -> bool,
    P: Pipe,
{
    pipe: P,
    filter: F,
}

impl <F, P> FilterPipe<F, P>
where
    F: Fn(&<P as Pipe>::Item) -> bool,
    P: Pipe,
{
    pub fn new(pipe: P, filter: F) -> Self {
        Self {
            pipe: pipe,
            filter: filter,
        }
    }
}

impl <F, P> Pipe for FilterPipe<F, P>
where
    F: Fn(&<P as Pipe>::Item) -> bool,
    P: Pipe,
{
    type Item = <P as Pipe>::Item;

    fn merge_in(&self, input: Self::Item) {
        if (self.filter)(&input) {
            self.pipe.merge_in(input);
        }
    }
}

impl <F, P> PipeConstructor for FilterPipe<F, P>
where
    F: Fn(&<P as Pipe>::Item) -> bool,
    P: Pipe,
{
    type Pipe = P;
    type Args = F;

    fn new(pipe: P, args: F) -> Self {
        Self::new(pipe, args)
    }
}
