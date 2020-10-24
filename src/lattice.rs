use crate::merge::Merge;


// LATTICE STRUCT //

pub struct Semilattice<T, F: Merge<T>> {
    val: T,
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

    // DANGER: Consumes this lattice, revealing it's value.
    pub fn into_reveal(self) -> T {
        self.val
    }
}

// Not important: lets you do `Default::default()`.
impl <T: Default, F: Merge<T>> Default for Semilattice<T, F> {
    fn default() -> Self {
        Self {
            val: Default::default(),
            _phantom: std::marker::PhantomData,
        }
    }
}

// Not important: lets you do `let x: Semilattice<_, ...> = something.into()`.
impl <T, F: Merge<T>> From<T> for Semilattice<T, F> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}
