use super::Pipe;


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