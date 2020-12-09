use crate::merge::Merge;


// LATTICE STRUCT //

pub struct Semilattice<F: Merge> {
    val: F::Domain,
}

impl <F: Merge> Semilattice<F> {
    pub fn new(val: F::Domain) -> Self {
        Self {
            val: val,
        }
    }

    pub fn merge_in(&mut self, val: F::Domain) {
        F::merge_in(&mut self.val, val);
    }

    // DANGER: Consumes this lattice, revealing it's value.
    pub fn into_reveal(self) -> F::Domain {
        self.val
    }
    // DANGER
    pub fn reveal(&self) -> &F::Domain {
        &self.val
    }
}

impl <F: Merge> Default for Semilattice<F>
where
    F::Domain: Default,
{
    fn default() -> Self
    {
        Self::new(F::Domain::default())
    }
}

impl <F: Merge> Clone for Semilattice<F>
where
    F::Domain: Clone,
{
    fn clone(&self) -> Self {
        Semilattice::new(self.val.clone())
    }
}
