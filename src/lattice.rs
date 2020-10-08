use crate::merge::Merge;

pub struct Lattice<T, F: Merge<T>> {
    val: T,
    merger: std::marker::PhantomData<F>,
}

impl <T, F: Merge<T>> Lattice<T, F> {
    pub fn new(val: T) -> Self {
        Lattice {
            val: val,
            merger: std::marker::PhantomData,
        }
    }

    pub fn merge(&self, other: &Self) {
        F::merge(&self.val, &other.val)
    }

    pub fn reveal(&self) -> &T {
        &self.val
    }

    pub fn into_reveal(self) -> T {
        self.val
    }
}

// Not important: lets you do `Lattice::default()`.
impl <T: Default, F: Merge<T>> Default for Lattice<T, F> {
    fn default() -> Self {
        Lattice {
            val: Default::default(),
            merger: std::marker::PhantomData,
        }
    }
}

// Not important: lets you do `let x: Lattice = something.into()`.
impl <T, F: Merge<T>> From<T> for Lattice<T, F> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}
