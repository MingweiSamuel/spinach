//! All the standard operators.

mod optrait;
pub use optrait::*;

mod nullop;
pub use nullop::*;

mod cloneop;
pub use cloneop::*;

mod latticeop;
pub use latticeop::*;

mod splitop;
pub use splitop::*;