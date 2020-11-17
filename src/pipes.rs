use std::fmt::Debug;

use tokio::sync::mpsc;
use tokio::sync::broadcast;

// use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;


pub trait Pipe<'a> {
    type Input;
    type Output;

    fn push(&'a mut self, item: Self::Input) -> Self::Output;
}

pub struct DebugPipe<T: Debug> {
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Debug> DebugPipe<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl <'a, T: Debug> Pipe<'a> for DebugPipe<T> {
    type Input = T;
    type Output = T;

    fn push(&'a mut self, item: T) -> T {
        println!("{:#?}", item);
        item
    }
}


pub struct ClonePipe<'b, T> {
    _phantom: std::marker::PhantomData<&'b T>,
}
impl <'a, 'b, T> Pipe<'a> for ClonePipe<'b, T>
where
    T: Clone,
{
    type Input = &'b T;
    type Output = T;

    fn push(&'a mut self, item: Self::Input) -> Self::Output {
        item.clone()
    }
}

pub struct MpscPipe<T> {
    sender: mpsc::Sender<T>,
}
impl <T> MpscPipe<T> {
    pub fn new(sender: mpsc::Sender<T>) -> Self {
        Self {
            sender: sender,
        }
    }
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

pub struct BroadcastPipe<T: Clone> {
    sender: broadcast::Sender<T>,
}
impl <T: Clone> BroadcastPipe<T> {
    pub fn new(sender: broadcast::Sender<T>) -> Self {
        Self {
            sender: sender,
        }
    }
}
impl <'a, T: Clone> Pipe<'a> for BroadcastPipe<T> {
    type Input = T;
    type Output = Result<usize, broadcast::error::SendError<T>>;

    fn push(&'a mut self, item: Self::Input) -> Self::Output {
        self.sender.send(item)
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
        let item = self.pipe_a.push(item);
        self.pipe_b.push(item)
    }
}


pub struct AnnaWorker<'a, 'b, F, P>
where
    F: Merge,
    P: Pipe<'a, Input = &'b F::Domain>,
    F::Domain: 'b,
{
    value: F::Domain,
    delta_receiver: mpsc::Receiver<F::Domain>,
    pipe_receiver: mpsc::Receiver<P>,
    pipes: Vec<P>,

    _phantom: std::marker::PhantomData<&'a ()>,
}
impl <'a, 'b: 'a, F, P> AnnaWorker<'a, 'b, F, P>
where
    F: Merge,
    P: Pipe<'a, Input = &'b F::Domain>,
    F::Domain: 'b,
{
    pub fn create(bottom: F::Domain) -> ( MpscPipe<F::Domain>, MpscPipe<P>, Self ) {
        let ( delta_sender, delta_receiver ) = mpsc::channel(16);
        let ( pipe_sender, pipe_receiver ) = mpsc::channel(16);

        let delta_sender = MpscPipe::new(delta_sender);
        let pipe_sender = MpscPipe::new(pipe_sender);

        let worker = Self {
            value: bottom,
            delta_receiver: delta_receiver,
            pipe_receiver: pipe_receiver,
            pipes: Vec::new(),

            _phantom: std::marker::PhantomData,
        };
        ( delta_sender, pipe_sender, worker )
    }

    // pub async fn run<'b: 'a>(&'b mut self) {
    //     loop {
    //         tokio::select! {
    //             Some(delta) = self.delta_receiver.recv() => {
    //                 F::merge_in(&mut self.value, delta);
    //                 for pipe in &mut self.pipes {
    //                     pipe.push(&self.value);
    //                 }
    //             },
    //             Some(pipe) = self.pipe_receiver.recv() => {
    //                 self.pipes.push(pipe);
    //                 self.pipes.last_mut().unwrap().push(&self.value);
    //             },
    //         }
    //     }
    // }

    pub fn tick(&'b mut self) {
        while let Ok(delta) = self.delta_receiver.try_recv() {
            F::merge_in(&mut self.value, delta);
        }
        while let Ok(pipe) = self.pipe_receiver.try_recv() {
            self.pipes.push(pipe);
        }
        for pipe in &mut self.pipes {
            let _result = pipe.push(&self.value);
        }
    }
}

#[tokio::test]
async fn test_asdf() {
    use std::collections::HashSet;

    use crate::merge;

    let ( mut delta_sender, mut pipe_sender, mut worker ) =
        AnnaWorker::<'_, '_, merge::Union<HashSet<usize>>, _>::create(HashSet::new());

    {
        delta_sender.push(vec![ 1_usize, 2, 3 ].into_iter().collect()).await;
        worker.tick();
    }

    {
        pipe_sender.push(DebugPipe::new()).await;
        worker.tick();
    }

    {
        delta_sender.push(vec![ 3, 4, 6 ].into_iter().collect()).await;
        worker.tick();
    }
}
