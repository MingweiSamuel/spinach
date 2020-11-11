use tokio::sync::mpsc;

use crate::merge::Merge;
use crate::semilattice::Semilattice;
use crate::traits::{ Set, SetUnion };

pub trait Pipe {
    type Input;

    fn push(&mut self, item: Self::Input);
}

pub struct Sink<F: Merge>
{
    semilattice: Semilattice<F>,
    send_senders: mpsc::Sender<mpsc::Sender<F::Domain>>,
    receive_senders: mpsc::Receiver<mpsc::Sender<F::Domain>>,
    senders: Vec<mpsc::Sender<F::Domain>>,
    _phantom: std::marker::PhantomData<F>,
}
impl <F: Merge> Sink<F> {
    pub fn new(bottom: F::Domain) -> Self {
        let ( send_senders, receive_senders ) = mpsc::channel(16);
        Self {
            semilattice: Semilattice::new(bottom),
            send_senders: send_senders,
            receive_senders: receive_senders,
            senders: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl <X> Sink<SetUnion<X>>
where
    X: Set + Extend<<X as Set>::Domain> + IntoIterator<Item = <X as Set>::Domain> + Clone,
{
    pub fn source(&mut self) -> mpsc::Receiver<X> {
        let ( sender, receiver ) = mpsc::channel::<X>(1);
        sender.try_send(self.semilattice.reveal().clone()).ok().expect("Failed to send to new channel.");
        self.senders.push(sender);
        receiver
    }
}

impl <F: Merge> Pipe for Sink<F>
where
    F::Domain: Clone,
{
    type Input = F::Domain;

    fn push(&mut self, item: F::Domain) {
        while let Ok(sender) = self.receive_senders.try_recv() {
            sender.try_send(self.semilattice.reveal().clone())
                .ok().expect("Failed to send to new channel.");
            self.senders.push(sender);
        }
        for sender in self.senders.iter() {
            sender.send(item.clone());
        }
        self.semilattice.merge_in(item);
    }
}

