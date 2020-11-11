use std::collections::{ BTreeSet };
use std::sync::mpsc;

use crate::semilattice::Semilattice;
use crate::traits::{ SetUnion, SemilatticeMorphismFn };


pub struct Reactive<T, F>
where
    F: SemilatticeMorphismFn<DomainMerge = SetUnion<BTreeSet<T>>>
{
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
    all_els: BTreeSet<T>,
    _phantom: std::marker::PhantomData<F>,
}
impl <T: Ord + Clone, F> Reactive<T, F>
where
    F: SemilatticeMorphismFn<DomainMerge = SetUnion<BTreeSet<T>>>
{
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender: sender,
            receiver: receiver,
            all_els: BTreeSet::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn send(&self, el: T) -> Result<(), mpsc::SendError<T>> {
        self.sender.send(el)
    }

    pub fn tick(&mut self) {
        let el = self.receiver.try_recv().expect("oops");
        self.all_els.insert(el);

        // PUSH INTO PIPE
        // self.pipe.push(self.all_els.clone())
        let result = F::call(Semilattice::new(self.all_els.clone()));
        let _ = result;
    }

    // API: need to have user-exposed pipeline openings at the end.
}
