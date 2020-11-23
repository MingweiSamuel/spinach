use std::future::{ self, Future };
use std::fmt::Debug;
// use std::sync::mpsc;

use tokio::sync::mpsc;
// use tokio::sync::broadcast;

// use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;


pub trait UnaryFn<I> {
    type Output;

    fn call(&self, input: I) -> Self::Output;
}


pub trait Pipe<'s> {
    type Item;
    type Feedback: Future;

    #[must_use]
    fn push(&'s mut self, item: &Self::Item) -> Self::Feedback;
}

pub trait MovePipe<'s> {
    type Item;
    type Feedback: Future;

    #[must_use]
    fn push(&'s mut self, item: Self::Item) -> Self::Feedback;
}

impl<'s, P: Pipe<'s>> MovePipe<'s> for P {
    type Item = P::Item;
    type Feedback = P::Feedback;

    fn push(&'s mut self, item: Self::Item) -> Self::Feedback {
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
impl<'s, T> Pipe<'s> for NullPipe<T> {
    type Item = T;
    type Feedback = future::Ready<()>;

    fn push(&'s mut self, _item: &Self::Item) -> Self::Feedback {
        future::ready(())
    }
}


pub struct DebugPipe<'s, P: Pipe<'s>>
where
    P::Item: Debug,
{
    tag: &'s str,
    next_pipe: P,
}
impl<'s, P: Pipe<'s>> DebugPipe<'s, P>
where
    P::Item: Debug,
{
    pub fn new(tag: &'s str, next_pipe: P) -> Self {
        Self {
            tag: tag,
            next_pipe: next_pipe,
        }
    }
}
impl<'s, P: Pipe<'s>> Pipe<'s> for DebugPipe<'s, P>
where
    P::Item: Debug,
{
    type Item = P::Item;
    type Feedback = P::Feedback;

    fn push(&'s mut self, item: &Self::Item) -> Self::Feedback {
        println!("{}: {:?}", self.tag, &item);
        self.next_pipe.push(item)
    }
}




pub struct ClonePipe<'s, P: MovePipe<'s>>
where
    P::Item: Clone,
{
    next_pipe: P,
    _phantom: std::marker::PhantomData<&'s ()>,
}
impl<'s, P: MovePipe<'s>> ClonePipe<'s, P>
where
    P::Item: Clone,
{
    pub fn new(next_pipe: P) -> Self {
        Self {
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<'s, P: MovePipe<'s>> Pipe<'s> for ClonePipe<'s, P>
where
    P::Item: Clone,
{
    type Item = P::Item;
    type Feedback = P::Feedback;

    fn push(&'s mut self, item: &Self::Item) -> Self::Feedback {
        self.next_pipe.push(item.clone())
    }
}




pub struct LatticePipe<'s, F: Merge, P: Pipe<'s, Item = F::Domain>> {
    value: F::Domain,
    next_pipe: P,
    _phantom: std::marker::PhantomData<&'s ()>,
}
impl<'s, F: Merge, P: Pipe<'s, Item = F::Domain>> LatticePipe<'s, F, P> {
    pub fn new(bottom: F::Domain, next_pipe: P) -> Self {
        Self {
            value: bottom,
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<'s, F: Merge, P: Pipe<'s, Item = F::Domain>> MovePipe<'s> for LatticePipe<'s, F, P> {
    type Item = F::Domain;
    type Feedback = P::Feedback;

    fn push(&'s mut self, item: Self::Item) -> Self::Feedback {
        F::merge_in(&mut self.value, item);
        self.next_pipe.push(&self.value)
    }
}


pub struct MpscPipe<T> {
    sender: mpsc::Sender<T>,
}
impl<T> MpscPipe<T> {
    pub fn create(sender: mpsc::Sender<T>) -> Self {
        Self {
            sender: sender,
        }
    }
}
impl<'s, T> MovePipe<'s> for MpscPipe<T> {
    type Item = T;
    type Feedback = impl Future;

    fn push(&'s mut self, item: T) -> Self::Feedback {
        self.sender.send(item)
    }
}
impl<T> Clone for MpscPipe<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}


// pub struct SplitPipe<P: MovePipe>
// where
//     P::Item: Clone,
// {
//     pipe_receiver: mpsc::Receiver<P>,
//     pipes: Vec<P>,
// }
// impl<P: Pipe> SplitPipe<P>
// where
//     P::Item: Clone,
// {
//     pub fn create() -> ( Self, MpscPipe<P> ) {
//         let ( sender, receiver ) = mpsc::sync_channel(8);
//         let inst = Self {
//             pipe_receiver: receiver,
//             pipes: Vec::new(),
//         };
//         let mpsc_pipe = MpscPipe::create(sender);
//         ( inst, mpsc_pipe )
//     }
// }
// impl<P: Pipe> Pipe for SplitPipe<P>
// where
//     P::Item: Clone,
// {
//     type Item = P::Item;

//     fn push(&mut self, item: &Self::Item) -> Result<(), String> {
//         while let Ok(new_pipe) = self.pipe_receiver.try_recv() {
//             self.pipes.push(new_pipe);
//         }
//         let mut result = Ok(());

//         self.pipes.drain_filter(|pipe| {
//             let next_result = pipe.push(item);
//             let remove = next_result.is_err(); // DANGER!!!! Errored pipes get removed!!
//             result = std::mem::replace(&mut result, Ok(())).and(next_result); // Ugly to fight ownership.
//             remove
//         });
//         result
//     }
// }


// pub struct MapFilterPipe<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: MovePipe> {
//     mapfilter: F,
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<T>,
// }
// impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: MovePipe> MapFilterPipe<T, F, P> {
//     pub fn new(mapfilter: F, next_pipe: P) -> Self {
//         Self {
//             mapfilter: mapfilter,
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl<T, F: for<'a> UnaryFn<&'a T, Output = Option<P::Item>>, P: MovePipe> Pipe for MapFilterPipe<T, F, P> {
//     type Item = T;

//     fn push(&mut self, item: &T) -> Result<(), String> {
//         if let Some(item) = self.mapfilter.call(item) {
//             self.next_pipe.push(item)
//         }
//         else {
//             Ok(())
//         }
//     }
// }
