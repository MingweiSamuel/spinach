use std::future; //::{ self, Future };
use std::fmt::Debug;
// use std::sync::mpsc;

use futures;
use futures::future::{ Either, FutureExt };

use tokio::sync::mpsc;
// use tokio::sync::broadcast;

// // use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;

use super::*;




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
}
impl<T> SharedRefPipe for NullPipe<T> {
    type Feedback = future::Ready<()>;

    fn push(&self, _item: &Self::Item) -> Self::Feedback {
        future::ready(())
    }
}
impl<T> ExclRefPipe for NullPipe<T> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: &Self::Item) -> Self::Feedback {
        future::ready(())
    }
}
impl<T> SharedMovePipe for NullPipe<T> {
    type Feedback = future::Ready<()>;

    fn push(&self, _item: Self::Item) -> Self::Feedback {
        future::ready(())
    }
}
impl<T> ExclMovePipe for NullPipe<T> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: Self::Item) -> Self::Feedback {
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
}
impl<P: SharedRefPipe> SharedRefPipe for DebugPipe<P>
where
    P::Item: Debug,
{
    type Feedback = P::Feedback;

    fn push(&self, item: &Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.next_pipe.push(item)
    }
}
impl<P: ExclRefPipe> ExclRefPipe for DebugPipe<P>
where
    P::Item: Debug,
{
    type Feedback = P::Feedback;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.next_pipe.push(item)
    }
}
impl<P: SharedMovePipe> SharedMovePipe for DebugPipe<P>
where
    P::Item: Debug,
{
    type Feedback = P::Feedback;

    fn push(&self, item: Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, &item);
        self.next_pipe.push(item)
    }
}
impl<P: ExclMovePipe> ExclMovePipe for DebugPipe<P>
where
    P::Item: Debug,
{
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, &item);
        self.next_pipe.push(item)
    }
}




pub struct ClonePipe<P: Pipe>
where
    P::Item: Clone,
{
    next_pipe: P,
}
impl<P: Pipe> ClonePipe<P>
where
    P::Item: Clone,
{
    pub fn new(next_pipe: P) -> Self {
        Self {
            next_pipe: next_pipe,
        }
    }
}
impl<P: Pipe> Pipe for ClonePipe<P>
where
    P::Item: Clone,
{
    type Item = P::Item;
}
impl<P: SharedMovePipe> SharedRefPipe for ClonePipe<P>
where
    P::Item: Clone,
{
    type Feedback = P::Feedback;

    fn push(&self, item: &Self::Item) -> Self::Feedback {
        self.next_pipe.push(item.clone())
    }
}
impl<P: ExclMovePipe> ExclRefPipe for ClonePipe<P>
where
    P::Item: Clone,
{
    type Feedback = P::Feedback;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        self.next_pipe.push(item.clone())
    }
}




pub struct LatticePipe<F: Merge, P: ExclRefPipe<Item = F::Domain>> {
    value: F::Domain,
    next_pipe: P,
}
impl<F: Merge, P: ExclRefPipe<Item = F::Domain>> LatticePipe<F, P> {
    pub fn new(bottom: F::Domain, next_pipe: P) -> Self {
        Self {
            value: bottom,
            next_pipe: next_pipe,
        }
    }
}
impl<F: Merge, P: ExclRefPipe<Item = F::Domain>> Pipe for LatticePipe<F, P> {
    type Item = F::Domain;
}
impl<F: Merge, P: ExclRefPipe<Item = F::Domain>> ExclMovePipe for LatticePipe<F, P> {
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
impl<T: 'static> Pipe for MpscPipe<T> {
    type Item = T;
}
impl<T: 'static> SharedMovePipe for MpscPipe<T> {
    type Feedback = impl Future;

    fn push(&self, item: T) -> Self::Feedback {
        // self.sender.send(item)
        // async {
        //     self.sender.send(item).await
        // }
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


pub struct SplitPipe<P: Pipe> {
    pipe_receiver: mpsc::Receiver<P>,
    pipes: Vec<P>,
}
impl<P: Pipe> SplitPipe<P> {
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
impl<P: Pipe> Pipe for SplitPipe<P> {
    type Item = P::Item;
}
impl<P: ExclRefPipe> ExclRefPipe for SplitPipe<P> {
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



pub struct MapFilterPipe<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: Pipe> {
    mapfilter: F,
    next_pipe: P,
    _phantom: std::marker::PhantomData<T>,
}
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: Pipe> MapFilterPipe<T, F, P> {
    pub fn new(mapfilter: F, next_pipe: P) -> Self {
        Self {
            mapfilter: mapfilter,
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: Pipe> Pipe for MapFilterPipe<T, F, P> {
    type Item = T;
}
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: SharedMovePipe> SharedRefPipe for MapFilterPipe<T, F, P> {
    type Feedback = impl Future;

    fn push(&self, item: &T) -> Self::Feedback {
        if let Some(item) = self.mapfilter.call(item) {
            Either::Left(self.next_pipe.push(item)
                .map(|x| Some(x)))
        }
        else {
            Either::Right(future::ready(None))
        }
    }
}
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: ExclMovePipe> ExclRefPipe for MapFilterPipe<T, F, P> {
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
