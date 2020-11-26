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




pub struct NullOp<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> NullOp<T> {
    pub fn new() -> Self {
        NullOp {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T> Op for NullOp<T> {
    type Item = T;
}
impl<T> SharedRefOp for NullOp<T> {
    type Feedback = future::Ready<()>;

    fn push(&self, _item: &Self::Item) -> Self::Feedback {
        future::ready(())
    }
}
impl<T> ExclRefOp for NullOp<T> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: &Self::Item) -> Self::Feedback {
        future::ready(())
    }
}
impl<T> SharedMoveOp for NullOp<T> {
    type Feedback = future::Ready<()>;

    fn push(&self, _item: Self::Item) -> Self::Feedback {
        future::ready(())
    }
}
impl<T> ExclMoveOp for NullOp<T> {
    type Feedback = future::Ready<()>;

    fn push(&mut self, _item: Self::Item) -> Self::Feedback {
        future::ready(())
    }
}


pub struct DebugOp<P: Op>
where
    P::Item: Debug,
{
    tag: &'static str,
    next_pipe: P,
}
impl<P: Op> DebugOp<P>
where
    P::Item: Debug,
{
    pub fn new(tag: &'static str, next_pipe: P) -> Self {
        DebugOp {
            tag: tag,
            next_pipe: next_pipe,
        }
    }
}
impl<P: Op> Op for DebugOp<P>
where
    P::Item: Debug,
{
    type Item = P::Item;
}
impl<P: SharedRefOp> SharedRefOp for DebugOp<P>
where
    P::Item: Debug,
{
    type Feedback = P::Feedback;

    fn push(&self, item: &Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.next_pipe.push(item)
    }
}
impl<P: ExclRefOp> ExclRefOp for DebugOp<P>
where
    P::Item: Debug,
{
    type Feedback = P::Feedback;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.next_pipe.push(item)
    }
}
impl<P: SharedMoveOp> SharedMoveOp for DebugOp<P>
where
    P::Item: Debug,
{
    type Feedback = P::Feedback;

    fn push(&self, item: Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, &item);
        self.next_pipe.push(item)
    }
}
impl<P: ExclMoveOp> ExclMoveOp for DebugOp<P>
where
    P::Item: Debug,
{
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, &item);
        self.next_pipe.push(item)
    }
}




pub struct CloneOp<P: Op>
where
    P::Item: Clone,
{
    next_pipe: P,
}
impl<P: Op> CloneOp<P>
where
    P::Item: Clone,
{
    pub fn new(next_pipe: P) -> Self {
        CloneOp {
            next_pipe: next_pipe,
        }
    }
}
impl<P: Op> Op for CloneOp<P>
where
    P::Item: Clone,
{
    type Item = P::Item;
}
impl<P: SharedMoveOp> SharedRefOp for CloneOp<P>
where
    P::Item: Clone,
{
    type Feedback = P::Feedback;

    fn push(&self, item: &Self::Item) -> Self::Feedback {
        self.next_pipe.push(item.clone())
    }
}
impl<P: ExclMoveOp> ExclRefOp for CloneOp<P>
where
    P::Item: Clone,
{
    type Feedback = P::Feedback;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        self.next_pipe.push(item.clone())
    }
}




pub struct LatticeOp<F: Merge, P: ExclRefOp<Item = F::Domain>> {
    value: F::Domain,
    next_pipe: P,
}
impl<F: Merge, P: ExclRefOp<Item = F::Domain>> LatticeOp<F, P> {
    pub fn new(bottom: F::Domain, next_pipe: P) -> Self {
        LatticeOp {
            value: bottom,
            next_pipe: next_pipe,
        }
    }
}
impl<F: Merge, P: ExclRefOp<Item = F::Domain>> Op for LatticeOp<F, P> {
    type Item = F::Domain;
}
impl<F: Merge, P: ExclRefOp<Item = F::Domain>> ExclMoveOp for LatticeOp<F, P> {
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Item) -> Self::Feedback {
        F::merge_in(&mut self.value, item);
        self.next_pipe.push(&self.value)
    }
}



pub struct MpscOp<T: 'static> {
    sender: mpsc::Sender<T>,
}
impl<T: 'static> MpscOp<T> {
    pub fn create(sender: mpsc::Sender<T>) -> Self {
        MpscOp {
            sender: sender,
        }
    }
}
impl<T: 'static> Op for MpscOp<T> {
    type Item = T;
}
impl<T: 'static> SharedMoveOp for MpscOp<T> {
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
impl<T: 'static> Clone for MpscOp<T> {
    fn clone(&self) -> Self {
        MpscOp {
            sender: self.sender.clone(),
        }
    }
}






pub struct TeeOp<P0: Op, P1: Op<Item = P0::Item>> {
    pipe0: P0,
    pipe1: P1,
}
impl<P0: Op, P1: Op<Item = P0::Item>> TeeOp<P0, P1> {
    pub fn new(pipe0: P0, pipe1: P1) -> Self {
        Self {
            pipe0: pipe0,
            pipe1: pipe1,
        }
    }
}
impl<P0: Op, P1: Op<Item = P0::Item>> Op for TeeOp<P0, P1> {
    type Item = P0::Item;
}
impl<P0: SharedRefOp, P1: Op<Item = P0::Item> + SharedRefOp> SharedRefOp for TeeOp<P0, P1> {
    type Feedback = impl Future;

    fn push(&self, item: &Self::Item) -> Self::Feedback {
        futures::future::join(
            self.pipe0.push(item),
            self.pipe1.push(item),
        )
    }
}
impl<P0: ExclRefOp, P1: Op<Item = P0::Item> + ExclRefOp> ExclRefOp for TeeOp<P0, P1> {
    type Feedback = impl Future;

    fn push(&mut self, item: &Self::Item) -> Self::Feedback {
        futures::future::join(
            self.pipe0.push(item),
            self.pipe1.push(item),
        )
    }
}






pub struct SplitOp<P: Op> {
    pipe_receiver: mpsc::Receiver<P>,
    pipes: Vec<P>,
}
impl<P: Op> SplitOp<P> {
    pub fn create() -> ( Self, MpscOp<P> ) {
        let ( sender, receiver ) = mpsc::channel(8);
        let inst = SplitOp {
            pipe_receiver: receiver,
            pipes: Vec::new(),
        };
        let mpsc_pipe = MpscOp::create(sender);
        ( inst, mpsc_pipe )
    }
}
impl<P: Op> Op for SplitOp<P> {
    type Item = P::Item;
}
impl<P: ExclRefOp> ExclRefOp for SplitOp<P> {
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



pub struct MapFilterOp<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: Op> {
    mapfilter: F,
    next_pipe: P,
    _phantom: std::marker::PhantomData<T>,
}
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: Op> MapFilterOp<T, F, P> {
    pub fn new(mapfilter: F, next_pipe: P) -> Self {
        MapFilterOp {
            mapfilter: mapfilter,
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: Op> Op for MapFilterOp<T, F, P> {
    type Item = T;
}
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: SharedMoveOp> SharedRefOp for MapFilterOp<T, F, P> {
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
impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: ExclMoveOp> ExclRefOp for MapFilterOp<T, F, P> {
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
