// Re-export tokio.
pub use tokio;

// Keep merge as a module.
pub mod merge;

// Stream.merge_into trait fn.
mod merge_into_lattice;
pub use merge_into_lattice::*;

// Bring lattice to the top level.
mod lattice;
pub use lattice::*;
