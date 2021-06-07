//! All the standard operators.

mod optrait;
pub use optrait::*;

mod nullop;
pub use nullop::*;

mod constop;
pub use constop::*;

mod onceop;
pub use onceop::*;

mod iterop;
pub use iterop::*;

mod debugop;
pub use debugop::*;

mod latticeop;
pub use latticeop::*;

mod splitop;
pub use splitop::*;

mod mergeop;
pub use mergeop::*;

mod morphop;
pub use morphop::*;

mod stdinop;
pub use stdinop::*;

mod zipop;
pub use zipop::*;

mod tcpop;
pub use tcpop::*;

mod batchconvertop;
pub use batchconvertop::*;

mod symhashjoinop;
pub use symhashjoinop::*;