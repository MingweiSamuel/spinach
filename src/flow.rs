//! Flow types, either [`Rx`] or [`Df`].

/// Trait for types representing different types of flows, either dataflow or reactive.
///
/// This trait is sealed and cannot be implemented for types outside this crate.
pub trait Flow: private::Sealed {}

/// Flow representing a dataflow of distinct `T` values.
pub struct Df {
    _private: (),
}

/// Flow representing a reactive pipeline of a monotonically growing `F::Domain` value.
/// "Monotonically growing" order is determined by the [`Lattice`] function `F`.
pub struct Rx {
    _private: (),
}

impl Flow for Df {}
impl Flow for Rx {}

mod private {
    use super::*;

    pub trait Sealed {}

    // Implement for those same types, but no others.
    impl Sealed for Df {}
    impl Sealed for Rx {}
}
