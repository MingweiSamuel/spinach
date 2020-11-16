use tokio::sync::mpsc;
use tokio::sync::watch;

use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;


pub trait Pipe<'a> {
    type Input;
    type Output;

    fn push(&'a mut self, item: Self::Input) -> Result<Self::Output, &'static str>;
}

pub struct Tank<F: Merge> {
    value: F::Domain,
}

impl <'a, F: Merge> Pipe<'a> for Tank<F>
where
    F::Domain: 'a,
{
    type Input = F::Domain;
    type Output = &'a F::Domain;

    fn push(&'a mut self, item: Self::Input) -> Result<Self::Output, &'static str> {
        F::merge_in(&mut self.value, item);
        Ok(&self.value)
    }
}

pub struct PipeActor<'a, P: Pipe<'a>> {
    pipe: P,
    receiver: mpsc::Receiver<P::Input>,
}
impl <'a, P: Pipe<'a>> PipeActor<'a, P> {
    pub fn create(pipe: P) -> ( mpsc::Sender<P::Input>, Self ) {
        let ( sender, receiver ) = mpsc::channel(16);
        let inst = Self {
            pipe: pipe,
            receiver: receiver,
        };
        ( sender, inst )
    }

    pub async fn run<'b: 'a>(&'b mut self) {
        while let Some(item) = self.receiver.recv().await {
            'a: {
                let pipe = &'a mut self.pipe;
                let value = pipe.push(item).expect("Failed to get value in PipeActor pipe.");
            }
        }
    }
}

