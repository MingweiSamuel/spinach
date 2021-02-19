use crate::merge::Merge;

pub struct DF<T> {
    _private: T,
}
pub struct RX<F: Merge> {
    _private: F::Domain,
}

/// This trait is sealed and cannot be implemented for types outside this crate.
pub trait Flow: private::Sealed {
    type Domain;
}

impl<T> Flow for DF<T> {
    type Domain = T;
}
impl<F: Merge> Flow for RX<F> {
    type Domain = F::Domain;
}

mod private {
    use super::*;

    pub trait Sealed {}

    // Implement for those same types, but no others.
    impl<T> Sealed for DF<T> {}
    impl<F: Merge> Sealed for RX<F> {}
}
