use super::{ PipeConstructor, Pipe };

/// A pipe which maps elements.
pub struct MapPipe<T, F, P>
where
    F: Fn(T) -> <P as Pipe>::Item,
    P: Pipe,
{
    pipe: P,
    mapper: F,
    _phantom: std::marker::PhantomData<T>,
}

impl <T, F, P> MapPipe<T, F, P>
where
    F: Fn(T) -> <P as Pipe>::Item,
    P: Pipe,
{
    pub fn new(pipe: P, mapper: F) -> Self {
        Self {
            pipe: pipe,
            mapper: mapper,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl <T, F, P> Pipe for MapPipe<T, F, P>
where
    F: Fn(T) -> <P as Pipe>::Item,
    P: Pipe,
{
    type Item = T;

    fn merge_in(&self, input: T) {
        let input = (self.mapper)(input);
        self.pipe.merge_in(input);
    }
}

impl <T, F, P> PipeConstructor for MapPipe<T, F, P>
where
    F: Fn(T) -> <P as Pipe>::Item,
    P: Pipe,
{
    type Pipe = P;
    type Args = F;

    fn new(pipe: P, args: F) -> Self {
        Self::new(pipe, args)
    }
}
