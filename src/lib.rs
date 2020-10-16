// Re-export tokio.
pub use tokio;

// Bring lattice to the top level.
mod lattice;
pub use lattice::*;

// Keep merge as a module.
pub mod merge;

// Stream.merge_into trait fn.
mod merge_into_lattice;
pub use merge_into_lattice::*;

pub mod pipe;


