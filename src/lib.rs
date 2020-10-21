use std::collections::HashMap;
use std::hash::Hash;

/// A bag (multiset).
/// Keeps a count, does not keep extra copies.
pub struct Bag<T: Hash + Eq> {
    len: usize,
    tbl: HashMap<T, usize>,
}

impl <T: Hash + Eq> Bag<T> {
    pub fn new() -> Self {
        Self {
            len: 0,
            tbl: Default::default(),
        }
    }

    /// Gets the total size of the bag.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Inserts a single value into the bag.
    pub fn insert(&mut self, item: T) {
        self.len += 1;
        self.tbl.entry(item)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    /// Returns how many item the bag contains. Zero means it is not contained.
    pub fn contains(&self, item: &T) -> usize {
        self.tbl.get(item).cloned().unwrap_or(0)
    }
}

pub trait Morphism {
    type Input;
    type Output;

    fn transduce(&self, input: Self::Input) -> Self::Output;
}

pub struct Pipe {
}
