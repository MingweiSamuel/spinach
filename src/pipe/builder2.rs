use super::{ Pipe, MapPipe };

pub struct Builder<A> {
    prev_builder: A,
}
impl <A> Builder<A> {
    // pub fn new() -> Self {
    //     Builder {
    //         value: (),
    //     }
    // }

    // pub fn map<T, F, P>() -> Builder<MapPipeBuilder>
    // where
    //     F: Fn(T) -> <P as Pipe>::Item,
    //     P: Pipe,
    // {

    // }
}

trait PipeBuilder {
    fn connect<Q: Pipe>(pipe: Q) -> <Self as PipeBuilderGat<Q>>::Output
    where
        Self: PipeBuilderGat<Q>;
}

trait PipeBuilderGat<Q> {
    type Output;
}

impl <Q, F, A> PipeBuilderGat<Q> for MapPipeBuilder<F, A, <Q as Pipe>::Item>
where
    Q: Pipe,
    F: Fn(A) -> <Q as Pipe>::Item,
{
    type Output = MapPipe<A, F, Q>;
}


struct MapPipeBuilder<F, A, B>
where
    F: Fn(A) -> B,
{
    mapper: F,
    _phantom: std::marker::PhantomData<( A, B )>,
}
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
