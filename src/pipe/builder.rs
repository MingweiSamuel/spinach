use super::{ Pipe, PipeConstructor, Unconnected };

// pub struct Builder<P> {
//     _phantom: std::marker::PhantomData<P>,
// }

// impl <P> Builder<P> {
//     pub fn finish<Q: Pipe>(self, pipe: Q) {
//     }
// }

pub struct ChainedUnconnected<P, Q>
where
    P: Unconnected<NextPipe = <Q as Unconnected>::ThisPipe>,
    Q: Unconnected,
{
    first: P,
    second: Q,
}
impl <P, Q> ChainedUnconnected<P, Q>
where
    P: Unconnected<NextPipe = <Q as Unconnected>::ThisPipe>,
    Q: Unconnected,
{
    pub fn new(first: P, second: Q) -> Self {
        Self {
            first: first,
            second: second,
        }
    }
}
impl <P, Q> Unconnected for ChainedUnconnected<P, Q>
where
    P: Unconnected<NextPipe = <Q as Unconnected>::ThisPipe>,
    Q: Unconnected,
{
    type ThisPipe = <P as Unconnected>::ThisPipe;
    type NextPipe = <Q as Unconnected>::NextPipe;

    fn connect(self, next_pipe: Self::NextPipe) -> Self::ThisPipe {
        let next_pipe = self.second.connect(next_pipe);
        self.first.connect(next_pipe)
    }
}



    // type ThisPipe = C;
    // type NextPipe = <C as PipeConstructor>::Pipe;
    // type Args = <C as PipeConstructor>::Args;

// struct ChainedPipeConstructor<P, Q>
// where
//     P: PipeConstructor<Pipe = Q>,
//     Q: PipeConstructor,
// {
//     first: P,
//     first_args: <P as PipeConstructor>::Args,
//     second: Q,
// }
// impl <P, Q> PipeConstructor for ChainedPipeConstructor<P, Q>
// where
//     P: PipeConstructor<Pipe = Q>,
//     Q: PipeConstructor,
// {
//     type Pipe = <Q as PipeConstructor>::Pipe;
//     type Args = <Q as PipeConstructor>::Args;

//     fn new(pipe: Self::Pipe, args: Self::Args) -> P {
//         P::new(Q::new(pipe, args), self.first_args)
//     }
// }



// pub struct Builder<P> {
//     chain: P,
// }

// impl Builder<()> {
//     pub fn new() -> Self {
//         Self {
//             chain: (),
//         }
//     }
//     pub fn join<P: PipeConstructor>(self, args: <P as PipeConstructor>::Args) -> Builder<<P as PipeConstructor>::Args> {
//         Builder {
//             chain: args,
//         }
//     }
// }


// impl <P, Q> Builder<(P, Q)>
// where
//     P: PipeConstructor<Pipe = Q>,
//     Q: PipeConstructor,
// {
//     pub fn join(pipe: Q, args: <P as PipeConstructor>::Args) -> Builder<(P, Q)> {
//         Builder {
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }

// impl <P, Q> Builder<(P, Q)>
// where
//     P: PipeConstructor<Pipe = Q>,
//     Q: Pipe,
// {
//     pub fn finish(pipe: Q, args: <P as PipeConstructor>::Args) -> P {
//         P::new(pipe, args)
//     }
// }
