pub struct DF {
    _private: (),
}
pub struct RX {
    _private: (),
}

/// This trait is sealed and cannot be implemented for types outside this crate.
pub trait Flow: private::Sealed {}

impl Flow for DF {}
impl Flow for RX {}

mod private {
    pub trait Sealed {}

    // Implement for those same types, but no others.
    impl Sealed for super::DF {}
    impl Sealed for super::RX {}
}
