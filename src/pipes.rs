use std::future::{ self, Future };
use std::fmt::Debug;
// use std::sync::mpsc;

use futures;
use futures::future::{ Either, FutureExt };

use tokio::sync::mpsc;
// use tokio::sync::broadcast;

// use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;


pub trait UnaryFn<I> {
    type Output;

    fn call(&self, input: I) -> Self::Output;
}


pub trait Pipe {
    type Item;
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: &Self::Item) -> Self::Feedback;
}

pub trait MovePipe {
    type Item;
    type Feedback: Future;

    #[must_use]
    fn push(&mut self, item: Self::Item) -> Self::Feedback;
}

impl<P: Pipe> MovePipe for P {
    type Item = P::Item;
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Item) -> Self::Feedback {
        Pipe::push(self, &item)
    }
}




pub struct NullPipe<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> NullPipe<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T> Pipe for NullPipe<T> {
    type Item = T;
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: &Self::Item) -> Self::Feedback {
        future::ready(())
    }
}


pub struct DebugPipe<P: Pipe>
where
    P::Item: Debug,
{
    tag: &'static str,
    next_pipe: P,
}
impl<P: Pipe> DebugPipe<P>
where
    P::Item: Debug,
{
    pub fn new(tag: &'static str, next_pipe: P) -> Self {
        Self {
            tag: tag,
            next_pipe: next_pipe,
        }
    }
}
impl<P: Pipe> Pipe for DebugPipe<P>
where
    P::Item: Debug,
{
    type Item = P::Item;
    type Feedback = P::Feedback;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, &item);
        self.next_pipe.push(item)
    }
}




pub struct ClonePipe<P: MovePipe>
where
    P::Item: Clone,
{
    next_pipe: P,
}
impl<P: MovePipe> ClonePipe<P>
where
    P::Item: Clone,
{
    pub fn new(next_pipe: P) -> Self {
        Self {
            next_pipe: next_pipe,
        }
    }
}
impl<P: MovePipe> Pipe for ClonePipe<P>
where
    P::Item: Clone,
{
    type Item = P::Item;
    type Feedback = P::Feedback;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        self.next_pipe.push(item.clone())
    }
}




pub struct LatticePipe<F: Merge, P: Pipe<Item = F::Domain>> {
    value: F::Domain,
    next_pipe: P,
}
impl<F: Merge, P: Pipe<Item = F::Domain>> LatticePipe<F, P> {
    pub fn new(bottom: F::Domain, next_pipe: P) -> Self {
        Self {
            value: bottom,
            next_pipe: next_pipe,
        }
    }
}
impl<F: Merge, P: Pipe<Item = F::Domain>> MovePipe for LatticePipe<F, P> {
    type Item = F::Domain;
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Item) -> Self::Feedback {
        F::merge_in(&mut self.value, item);
        self.next_pipe.push(&self.value)
    }
}


pub struct MpscPipe<T: 'static> {
    sender: mpsc::Sender<T>,
}
impl<T: 'static> MpscPipe<T> {
    pub fn create(sender: mpsc::Sender<T>) -> Self {
        Self {
            sender: sender,
        }
    }
}
impl<T: 'static> MovePipe for MpscPipe<T> {
    type Item = T;
    type Feedback = impl Future;

    fn push(&mut self, item: T) -> Self::Feedback {
        let sender = self.sender.clone();
        async move {
            sender.send(item).await
        }
    }
}
impl<T: 'static> Clone for MpscPipe<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}


pub struct SplitPipe<P: MovePipe>
where
    P::Item: Clone,
{
    pipe_receiver: mpsc::Receiver<P>,
    pipes: Vec<P>,
}
impl<P: Pipe> SplitPipe<P>
where
    P::Item: Clone,
{
    pub fn create() -> ( Self, MpscPipe<P> ) {
        let ( sender, receiver ) = mpsc::channel(8);
        let inst = Self {
            pipe_receiver: receiver,
            pipes: Vec::new(),
        };
        let mpsc_pipe = MpscPipe::create(sender);
        ( inst, mpsc_pipe )
    }
}
impl<P: Pipe> Pipe for SplitPipe<P>
where
    P::Item: Clone,
{
    type Item = P::Item;
    type Feedback = impl Future;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        while let Ok(new_pipe) = self.pipe_receiver.try_recv() {
            self.pipes.push(new_pipe);
        }

        let pushes = self.pipes
            .iter_mut()
            .map(|pipe| pipe.push(item));
        futures::future::join_all(pushes)
    }
}


pub struct MapFilterPipe<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: MovePipe> {
    mapfilter: F,
    next_pipe: P,
    _phantom: std::marker::PhantomData<T>,
}
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: MovePipe> MapFilterPipe<T, F, P> {
    pub fn new(mapfilter: F, next_pipe: P) -> Self {
        Self {
            mapfilter: mapfilter,
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: MovePipe> Pipe for MapFilterPipe<T, F, P> {
    type Item = T;
    type Feedback = impl Future;

    fn push(&mut self, item: &T) -> Self::Feedback {
        if let Some(item) = self.mapfilter.call(item) {
            Either::Left(self.next_pipe.push(item)
                .map(|x| Some(x)))
        }
        else {
            Either::Right(future::ready(None))
        }
    }
}
