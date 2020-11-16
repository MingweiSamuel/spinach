use tokio::sync::mpsc;
use tokio::sync::watch;

use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;


pub trait Pipe<'a> {
    type Input;
    type Output;

    fn push(&'a mut self, item: Self::Input) -> Self::Output;
}

pub struct ClonePipe<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl <'a, T: 'a> Pipe<'a> for ClonePipe<T>
where
    T: Clone,
{
    type Input = &'a T;
    type Output = T;

    fn push(&'a mut self, item: Self::Input) -> Self::Output {
        item.clone()
    }
}

pub struct MpscPipe<T> {
    sender: mpsc::Sender<T>,
}
impl <'a, T> Pipe<'a> for MpscPipe<T> {
    type Input = T;
    type Output = impl std::future::Future<Output = Result<(), mpsc::error::SendError<T>>>;

    fn push(&'a mut self, item: Self::Input) -> Self::Output {
        self.sender.send(item)
    }
}
impl <T> Clone for MpscPipe<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

pub struct SequentialPipe<'a, A, B>
where
    A: Pipe<'a>,
    B: Pipe<'a, Input = A::Output>,
{
    pipe_a: A,
    pipe_b: B,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl <'a, A, B> SequentialPipe<'a, A, B>
where
    A: Pipe<'a>,
    B: Pipe<'a, Input = A::Output>,
{
    pub fn new(pipe_a: A, pipe_b: B) -> Self {
        Self {
            pipe_a: pipe_a,
            pipe_b: pipe_b,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl <'a, A, B> Pipe<'a> for SequentialPipe<'a, A, B>
where
    A: Pipe<'a>,
    B: Pipe<'a, Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    fn push(&'a mut self, item: Self::Input) -> Self::Output {
        self.pipe_b.push(self.pipe_a.push(item))
    }
}


pub struct AnnaWorker<F: Merge> {
    value: F::Domain,
}
