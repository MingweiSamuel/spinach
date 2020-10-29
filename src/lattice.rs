use crate::merge::Merge;


// LATTICE STRUCT //
#[derive(Default)]
pub struct Semilattice<F>
where
    F: Merge,
{
    val: F::Domain,
}

impl <F> Semilattice<F>
where
    F: Merge,
{
    pub fn new(val: F::Domain) -> Self {
        Self {
            val: val,
        }
    }

    pub fn merge_in(&mut self, val: F::Domain) {
        F::merge(&mut self.val, val);
    }

    // DANGER: Consumes this lattice, revealing it's value.
    pub fn into_reveal(self) -> F::Domain {
        self.val
    }
}
