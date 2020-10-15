use std::future::Future;
use std::pin::Pin;

use tokio::stream::{ Stream, StreamExt };

use crate::Lattice;
use crate::merge::Merge;

pub trait MergeIntoLattice<T, F: Merge<T>> {
    fn merge_into(self, target: Lattice<T, F>) -> Pin<Box<dyn Future<Output = Lattice<T, F>> + 'static>>;
}

impl <T: 'static, F: Merge<T> + 'static, S: Stream<Item = T> + Unpin + 'static> MergeIntoLattice<T, F> for S {
    fn merge_into(self, mut target: Lattice<T, F>) -> Pin<Box<dyn Future<Output = Lattice<T, F>> + 'static>> {
        Box::pin(async move {
            let mut stream = self;
            while let Some(item) = stream.next().await {
                target.merge_in(item);
            };
            target
        })
    }
}
