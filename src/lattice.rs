use crate::merge::Merge;

// LATTICE STRUCT //

pub struct Lattice<T, F: Merge<T>> {
    val: T,
    _phantom: std::marker::PhantomData<F>,
}

impl <T, F: Merge<T>> Lattice<T, F> {
    pub fn new(val: T) -> Self {
        Lattice {
            val: val,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn merge_in(&mut self, val: T) {
        F::merge(&mut self.val, val);
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

// Not important: lets you do `Lattice::default()`.
impl <T: Default, F: Merge<T>> Default for Lattice<T, F> {
    fn default() -> Self {
        Lattice {
            val: Default::default(),
            _phantom: std::marker::PhantomData,
        }
    }
}

// Not important: lets you do `let x: Lattice = something.into()`.
impl <T, F: Merge<T>> From<T> for Lattice<T, F> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

// Not important: lets you debug print... (also kinda illegal).
impl <T: std::fmt::Debug, F: Merge<T>> std::fmt::Debug for Lattice<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.val.fmt(f)
    }
}
