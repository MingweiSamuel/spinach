//! All the standard operators.

mod optrait;
pub use optrait::*;

mod nullop;
pub use nullop::*;

mod debugop;
pub use debugop::*;

mod stdop;
pub use stdop::*;

mod mapfilterop;
pub use mapfilterop::*;

mod cloneop;
pub use cloneop::*;

mod referenceop;
pub use referenceop::*;

mod channelop;
pub use channelop::*;

mod handoffop;
pub use handoffop::*;

mod latticeop;
pub use latticeop::*;

mod monotonicop;
pub use monotonicop::*;

// mod explodeop;
// pub use explodeop::*;

mod timingop;
pub use timingop::*;

mod udpop;
pub use udpop::*;

mod naryop;
pub use naryop::*;

// mod rendezvousop;
// pub use rendezvousop::*;
