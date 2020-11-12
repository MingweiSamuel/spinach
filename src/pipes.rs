use tokio::sync::mpsc;
use tokio::sync::watch;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;

pub trait Pipe {
    type Input;

    fn push(&mut self, item: Self::Input);
}

pub fn create_tank<F: Merge>() {

}


pub struct Tank<F: Merge> {
    value: F::Domain,
    sender: watch::Sender<F::Domain>,
}

impl <F: Merge> Pipe for Tank<F>
where
    F::Domain: Clone,
{
    type Input = F::Domain;

    fn push(&mut self, item: Self::Input) {
        F::merge_in(&mut self.value, item);
        self.sender.send(self.value.clone());
    }
}