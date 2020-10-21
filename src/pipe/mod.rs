pub mod builder;
// pub mod builder2;

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
