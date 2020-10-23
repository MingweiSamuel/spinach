use std::cmp::Ordering;

use derivative::Derivative;

use crate::merge::Merge;


// LATTICE STRUCT //

#[derive(Derivative)]
#[derivative(Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Semilattice<T, F: Merge<T>> {
    val: T,

    // #[derivative(Debug="ignore")]
    // #[derivative(PartialEq="ignore")]
    // #[derivative(PartialOrd="ignore")]
    // #[derivative(Ord="ignore")]
    // #[derivative(Hash="ignore")]
    _phantom: std::marker::PhantomData<F>,
}

impl <T, F: Merge<T>> Semilattice<T, F> {
    pub fn new(val: T) -> Self {
        Self {
            val: val,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn merge_in(&mut self, val: T) {
        F::merge(&mut self.val, val);
    }

    pub fn reveal_partial_cmp(&self, other: &Self) -> Option<Ordering> {
        F::partial_cmp(&self.val, &other.val)
    }

    // DANGER: Reveals a shared reference to this lattice value.
    pub fn reveal(&self) -> &T {
        &self.val
    }

    // DANGER: Consumes this lattice, revealing it's value.
    pub fn into_reveal(self) -> T {
        self.val
    }
}

// // Not important: lets you do `Default::default()`.
// impl <T: Default, F: Merge<T>> Default for Semilattice<T, F> {
//     fn default() -> Self {
//         Self {
//             val: Default::default(),
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }

// Not important: lets you do `let x: Semilattice<_, ...> = something.into()`.
impl <T, F: Merge<T>> From<T> for Semilattice<T, F> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

// // Not important: lets you debug print... (also kinda illegal).
// impl <T: std::fmt::Debug, F: Merge<T>> std::fmt::Debug for Semilattice<T, F> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.val.fmt(f)
//     }
// }
