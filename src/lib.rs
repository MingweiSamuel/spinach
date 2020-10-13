// Re-export tokio.
pub use tokio;

// Keep merge as a module.
pub mod merge;

// Bring lattice to the top level.
mod lattice;
pub use lattice::*;

// For now use normal tokio async streams.
// TODO: Use custom version of Stream to limit operations to filter/map/fold.
pub use tokio::stream::{ Stream, StreamExt };
pub use tokio::sync::mpsc;


// STUFF THAT SHOULD BE IN A SEPARATE MODULE BELOW HERE.
use std::future::Future;
use std::pin::Pin;

use merge::Merge;

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
