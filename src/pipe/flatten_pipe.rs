use super::{ PipeConstructor, Pipe };

/// A pipe which flattens iterables.
pub struct FlattenPipe<T, P>
where
    T: IntoIterator,
    P: Pipe<Item = <T as IntoIterator>::Item>,
{
    pipe: P,
    _phantom: std::marker::PhantomData<T>,
}

impl <T, P> FlattenPipe<T, P>
where
    T: IntoIterator,
    P: Pipe<Item = <T as IntoIterator>::Item>,
{
    pub fn new(pipe: P) -> Self {
        Self {
            pipe: pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl <T, P> Pipe for FlattenPipe<T, P>
where
    T: IntoIterator,
    P: Pipe<Item = <T as IntoIterator>::Item>,
{
    type Item = T;

    fn merge_in(&self, input: T) {
        for item in input {
            self.pipe.merge_in(item);
        }
    }
}

impl <T, P> PipeConstructor for FlattenPipe<T, P>
where
    T: IntoIterator,
    P: Pipe<Item = <T as IntoIterator>::Item>,
{
    type Pipe = P;
    type Args = ();

    fn new(pipe: P, _args: ()) -> Self {
        Self::new(pipe)
    }
}
