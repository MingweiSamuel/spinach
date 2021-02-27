use ref_cast::RefCast;

use super::Lattice;

#[repr(transparent)]
#[derive(RefCast)]
pub struct Hide<F: Lattice>(F::Domain);

impl<F: Lattice> Hide<F> {
    pub fn new(val: F::Domain) -> Self {
        Hide(val)
    }
    pub fn reveal(&self) -> &F::Domain {
        &self.0
    }
    pub fn into_reveal(self) -> F::Domain {
        self.0
    }
}
