mod pipes;
pub use pipes::*;

use std::task::{ Context, Poll };

pub trait Source {
    type Output;

    /// Looks like an async stream.
    fn poll_next(&mut self, cx: &mut Context<'_>)
        -> Poll<Option<Self::Output>>;
}

pub trait Pipe {
    type Input;
    type Output;

    fn process(&self, input: Self::Input) -> Option<Self::Output>;
}

pub trait Sink {
    type Input;

    fn receive(&self, input: Self::Input);
}

pub trait Pipeflow {
}
