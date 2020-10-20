use super::{ Pipe, MapPipe, FilterPipe, FlattenPipe };

// pub struct Builder<A, PB> {
//     prev_builder: A,
//     pipe_builder: PB
// }
// impl <A> Builder<A> {
//     pub fn new(value: A) -> Self {
//         Self {
//             value: value,
//         }
//     }

//     // pub fn map<T, F, P>() -> Builder<MapPipeBuilder>
//     // where
//     //     F: Fn(T) -> <P as Pipe>::Item,
//     //     P: Pipe,
//     // {

//     // }

//     pub fn map<F, B>(self, mapper: F) -> Builder<B>
//     where
//         F: Fn(A) -> B
//     {
//         Builder {
//             prev_builder
//         }
//     }
// }

trait PipeBuilder<B> {
    fn connect<Q>(self, pipe: Q) -> <Self as PipeBuilderGat<Q>>::Output
    where
        Q: Pipe<Item = B>,
        Self: PipeBuilderGat<Q>;
}

trait PipeBuilderGat<Q> {
    type Output;
}


struct NoOpPipeBuilder;

impl <Q> PipeBuilderGat<Q> for NoOpPipeBuilder
where
    Q: Pipe,
{
    type Output = Q;
}

impl <B> PipeBuilder<B> for NoOpPipeBuilder {
    fn connect<Q>(self, pipe: Q) -> <Self as PipeBuilderGat<Q>>::Output
    where
        Q: Pipe<Item = B>
    {
        pipe
    }
}



struct MapPipeBuilder<F, A, B>
where
    F: Fn(A) -> B,
{
    mapper: F,
    _phantom: std::marker::PhantomData<( A, B )>,
}

impl <Q, F, A> PipeBuilderGat<Q> for MapPipeBuilder<F, A, <Q as Pipe>::Item>
where
    Q: Pipe,
    F: Fn(A) -> <Q as Pipe>::Item,
{
    type Output = MapPipe<A, F, Q>;
}

impl <F, A, B> PipeBuilder<B> for MapPipeBuilder<F, A, B>
where
    F: Fn(A) -> B,
{
    fn connect<Q>(self, pipe: Q) -> <Self as PipeBuilderGat<Q>>::Output
    where
        Q: Pipe<Item = B>,
    {
        MapPipe::new(pipe, self.mapper)
    }
}




struct FilterPipeBuilder<F, A>
where
    F: Fn(&A) -> bool,
{
    filter: F,
    _phantom: std::marker::PhantomData<A>,
}

impl <Q, F> PipeBuilderGat<Q> for FilterPipeBuilder<F, <Q as Pipe>::Item>
where
    Q: Pipe,
    F: Fn(&<Q as Pipe>::Item) -> bool,
{
    type Output = FilterPipe<F, Q>;
}

impl <F, A> PipeBuilder<A> for FilterPipeBuilder<F, A>
where
    F: Fn(&A) -> bool,
{
    fn connect<Q>(self, pipe: Q) -> <Self as PipeBuilderGat<Q>>::Output
    where
        Q: Pipe<Item = A>,
    {
        FilterPipe::new(pipe, self.filter)
    }
}



struct FlattenPipeBuilder<A>
where
    A: IntoIterator
{
    _phantom: std::marker::PhantomData<A>,
}

impl <Q, A> PipeBuilderGat<Q> for FlattenPipeBuilder<A>
where
    Q: Pipe<Item = <A as IntoIterator>::Item>,
    A: IntoIterator,
{
    type Output = FlattenPipe<A, Q>;
}

impl <A> PipeBuilder<<A as IntoIterator>::Item> for FlattenPipeBuilder<A>
where
    A: IntoIterator
{
    fn connect<Q>(self, pipe: Q) -> <Self as PipeBuilderGat<Q>>::Output
    where
        Q: Pipe<Item = <A as IntoIterator>::Item>,
    {
        FlattenPipe::new(pipe)
    }
}


// impl <F, A, P> PipeBuilder<MapPipe<A, F, P>> for MapPipeBuilder
// where
//     F: Fn(A) -> B,
//     P: Pipe
// {
//     type Output = B;
//     fn connect<P: Pipe<Item = Self::Output>>(self, pipe: P) -> MapPipe<A, F, P> {
//         MapPipe::new(pipe, self.mapper)
//     }
// }




// struct NoOpPipeBuilder<A> {
//     _phantom: std::marker::PhantomData<A>,
// }

// struct MapPipeBuilder<F, A, B>
// where
//     F: Fn(A) -> B,
// {
//     mapper: F,
//     _phantom: std::marker::PhantomData<( A, B )>,
// }
// impl <F, A, P> PipeBuilder<MapPipe<A, F, P>> MapPipeBuilder
// where
//     F: Fn(A) -> B,
//     P: Pipe
// {
//     type Output = B;
//     fn connect<P: Pipe<Item = Self::Output>>(self, pipe: P) -> MapPipe<A, F, P> {
//         MapPipe::new(pipe, self.mapper)
//     }
// }




// struct FilterPipeBuilder<F, A>
// where
//     F: Fn(A) -> bool,
// {
//     filter: F,
//     _phantom: std::marker::PhantomData<A>,
// }





// trait ConnectPipe<P: Pipe> {
//     fn connect<Q: Pipe>(next_pipe: Q) -> P;
// }
