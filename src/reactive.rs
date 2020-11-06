use std::collections::{ BTreeSet };
use std::sync::mpsc;
use std::thread;

use crate::semilattice::Semilattice;
use crate::traits::{ SetUnion, SemilatticeMorphismFn, UnaryFn };


pub trait Pipe {
    type Input;

    fn push(&mut self, item: Self::Input);
}


pub struct Reactive<T, P>
where
    P: Pipe<Input = BTreeSet<T>>,
{
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
    all_els: BTreeSet<T>,
    pipe: P,
}
impl <T: Ord + Clone, P> Reactive<T, P>
where
    P: Pipe<Input = BTreeSet<T>>,
{
    pub fn new(pipe: P) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender: sender,
            receiver: receiver,
            all_els: BTreeSet::new(),
            pipe: pipe,
        }
    }

    pub fn send(&self, el: T) {
        self.sender.send(el);
    }

    pub fn tick(&mut self) {
        let el = self.receiver.try_recv().expect("oops");
        self.all_els.insert(el);

        // PUSH INTO PIPE
        self.pipe.push(self.all_els.clone())
    }

    // API: need to have user-exposed pipeline openings at the end.
}
