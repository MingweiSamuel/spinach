//! Flow types, either [`Rx`] or [`Df`].

/// Trait for types representing different types of flows, either dataflow or reactive.
///
/// This trait is sealed and cannot be implemented for types outside this crate.
pub trait Flow: private::Sealed {
    type Domain;
}

/// Flow representing a dataflow of distinct `T` values.
pub struct Df<T> {
    _private: T,
}

/// Flow representing a reactive pipeline of a monotonically growing `F::Domain` value.
/// "Monotonically growing" order is determined by the [`Lattice`] function `F`.
pub struct Rx<T> {
    _private: T,
}

impl<T> Flow for Df<T> {
    type Domain = T;
}

impl<T> Flow for Rx<T> {
    type Domain = T;
}

mod private {
    use super::*;

    pub trait Sealed {}

    // Implement for those same types, but no others.
    impl<T> Sealed for Df<T> {}
    impl<T> Sealed for Rx<T> {}
}
