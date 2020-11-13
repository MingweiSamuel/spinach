use tokio::sync::mpsc;
use tokio::sync::watch;

use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;


pub trait UnaryFn {
    type Domain;
    type Codomain;

    fn call(input: Self::Domain) -> Self::Codomain;
}



pub trait Pipe {
    type Input;

    fn push(&mut self, item: Self::Input) -> Result<(), &'static str>;
}



pub struct Tank<F: Merge, P>
where
    P: Pipe<Input = F::Domain>
{

}



pub struct Unary<F: UnaryFn, P>
where
    P: Pipe<Input = F::Codomain>,
{
    pipe: P,
    _phantom: std::marker::PhantomData<F>,
}
impl <F: UnaryFn, P> Unary<F, P>
where
    P: Pipe<Input = F::Codomain>,
{
    pub fn new(pipe: P) -> Self {
        Self {
            pipe: pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl <F: UnaryFn, P> Pipe for Unary<F, P>
where
    P: Pipe<Input = F::Codomain>,
{
    type Input = F::Domain;

    fn push(&mut self, item: Self::Input) -> Result<(), &'static str> {
        self.pipe.push(F::call(item))
    }
}



// pub trait Pipe {
//     type Input;
//     type Output;

//     fn push(&mut self, item: Self::Input);

//     fn get<'a>(&'a mut self) -> &'a Self::Output;
// }

// pub struct Tank<F: Merge> {
//     value: F::Domain,
// }
// impl <F: Merge> Pipe for Tank<F> {
//     type Input = F::Domain;
//     type Output = F::Domain;

//     fn push(&mut self, item: Self::Input) {
//         F::merge_in(&mut self.value, item);
//     }

//     fn get<'a>(&'a mut self) -> &'a Self::Output {
//         &self.value
//     }
// }

// pub fn create_tank<F: Merge>() {

// }

// pub struct Tank<F: Merge> {
//     value: F::Domain,
//     sender: mpsc::Sender<F::Domain>,
//     receiver: mpsc::Receiver<F::Domain>,
// }

// pub struct SenderPipe<F: Merge>(mpsc::Sender<F::Domain>);
// impl <F: Merge> Pipe for SenderPipe<F> {
//     type Input = F::Domain;

//     fn push(&mut self, item: Self::Input) {
//         self.0.send(item);
//     }
// }


// pub struct Tank<F: Merge> {
//     value: F::Domain,
//     sender: watch::Sender<F::Domain>,
// }

// impl <F: Merge> Pipe for Tank<F>
// where
//     F::Domain: Clone,
// {
//     type Input = F::Domain;

//     fn push(&mut self, item: Self::Input) {
//         F::merge_in(&mut self.value, item);
//         self.sender.send(self.value.clone());

//         while let Ok(sender) = self.queue.try_recv() {
//             self.senders.push(sender);
//         }
//         for sender in &self.senders {
//             sender.send(self.value.clone());
//         }
//     }
// }



// pub struct Worker<M, S>
// where
//     M: Merge,
//     S: Stream<Item = M::Domain>,
// {
//     stream: S,
//     value: M::Domain,
//     _phantom: std::marker::PhantomData<M>,
// }

// impl <M, S> Worker<M, S>
// where
//     M: Merge,
//     S: Stream<Item = M::Domain>,
// {
//     pub fn new(stream: S, value: M::Domain) -> Self {
//         Self {
//             stream: stream,
//             value: value,
//             _phantom: std::marker::PhantomData,
//         }
//     }

//     async fn execute(&mut self) {

//     }
// }