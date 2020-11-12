// use tokio::sync::mpsc;

// use crate::merge::Merge;
// use crate::semilattice::Semilattice;

pub trait Pipe {
    type Input;

    fn push(&mut self, item: Self::Input);
}
