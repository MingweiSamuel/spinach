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
        F::merge(&mut self.val, val);
    }

    // DANGER: Consumes this lattice, revealing it's value.
    pub fn into_reveal(self) -> F::Domain {
        self.val
    }
}

// Not important: lets you do `Default::default()`.
impl <F: Merge> Default for Semilattice<F>
where
    F::Domain: Default
{
    fn default() -> Self {
        Self {
            val: Default::default(),
        }
    }
}

// // Not important: lets you do `let x: Semilattice<_, ...> = something.into()`.
// impl <T, F: Merge<Domain = T>> From<T> for Semilattice<F> {
//     fn from(val: T) -> Self {
//         Self::new(val)
//     }
// }
