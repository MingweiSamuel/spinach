use std::borrow::Borrow;

use super::Lattice;

pub struct Hide<F: Lattice> {
    val: F::Domain,
}

impl<F: Lattice> Hide<F> {
    pub fn new(val: F::Domain) -> Self {
        Self { val }
    }
    pub fn reveal(&self) -> &F::Domain {
        &self.val
    }
    pub fn into_reveal(self) -> F::Domain {
        self.val
    }
}

pub struct HideRef<'a, F: Lattice> {
    val: &'a F::Domain,
}

impl<'a, F: Lattice> HideRef<'a, F> {
    pub fn new(val: &'a F::Domain) -> Self {
        Self { val }
    }
    pub fn reveal(&self) -> &'a F::Domain {
        self.val
    }
}
