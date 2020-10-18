pub mod builder;

mod filter_pipe;
pub use filter_pipe::FilterPipe;

mod flatten_pipe;
pub use flatten_pipe::FlattenPipe;

mod map_pipe;
pub use map_pipe::MapPipe;

mod merge_pipe;
pub use merge_pipe::MergePipe;

mod split_pipe;
pub use split_pipe::SplitPipe;

mod tank;
pub use tank::Tank;


///
pub fn test() {
    use std::collections::HashMap;

    use crate::Lattice;
    use crate::merge::{ MaxMerge, MinMerge, DominatingPairMerge, MapUnionMerge };

    type VersionedString = Lattice<
        (Lattice<usize, MaxMerge>, Lattice<&'static str, MinMerge>),
        DominatingPairMerge>;
    type AnnaMap = Lattice<
        HashMap<&'static str, VersionedString>,
        MapUnionMerge>;

    let tank = Tank::new(AnnaMap::default());

    type Triple = ( &'static str, usize, &'static str );
    let builder = UnconnectedPipe::<FilterPipe<_, _>>::new(|( _, _, v ): &Triple| !v.contains("Christ"));
    let builder = builder::ChainedUnconnected::new(builder, UnconnectedPipe::<MapPipe<_, _, _>>::new(|( k, t, v )| {
        vec![ ( k, t, v ), ( k, t - 1, "other str" ) ]
    }));
    let builder = builder::ChainedUnconnected::new(builder, UnconnectedPipe::<FlattenPipe<_, _>>::new(()));
    let builder = builder::ChainedUnconnected::new(builder, UnconnectedPipe::<MapPipe<_, _, _>>::new(|( k, t, v ): Triple| {
        let mut y: HashMap<_, _> = Default::default();
        y.insert(k, ( t.into(), v.into() ).into());
        y
    }));

    let final_pipe = builder.connect(tank);
}



/// A pipe is something which can have items pushed into it.
pub trait Pipe {
    type Item;

    fn merge_in(&self, input: Self::Item);
}

pub trait Unconnected {
    type ThisPipe;
    type NextPipe;

    fn connect(self, next_pipe: Self::NextPipe) -> Self::ThisPipe;
}

pub struct UnconnectedPipe<C: PipeConstructor> {
    args: <C as PipeConstructor>::Args,
}
impl <C: PipeConstructor> UnconnectedPipe<C> {
    pub fn new(args: <C as PipeConstructor>::Args) -> Self {
        Self {
            args: args,
        }
    }
}
impl <C: PipeConstructor> Unconnected for UnconnectedPipe<C> {
    type ThisPipe = C;
    type NextPipe = <C as PipeConstructor>::Pipe;

    fn connect(self, next_pipe: Self::NextPipe) -> Self::ThisPipe {
        C::new(next_pipe, self.args)
    }
}

/// Trait for helping with pipe construction.
pub trait PipeConstructor {
    type Pipe;
    type Args;

    fn new(pipe: Self::Pipe, args: Self::Args) -> Self;
}

pub struct PipeConstructorStruct<C: PipeConstructor> {
    args: <C as PipeConstructor>::Args,
}
impl <C: PipeConstructor> PipeConstructorStruct<C> {
    pub fn new(args: <C as PipeConstructor>::Args) -> Self {
        Self {
            args: args,
        }
    }
    pub fn construct(self, pipe: <C as PipeConstructor>::Pipe) -> C {
        C::new(pipe, self.args)
    }
}
