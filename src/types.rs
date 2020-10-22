use std::collections::HashSet;
use std::hash::Hash;

// 1. What are filter and duplicate? From a morphism perspective.
//     .filter(): Item -> Option<Item> -> THEN FOLD JOIN (flatten).
//     .duplicate(): Item -> lots of items -> THEN FOLD JOIN.
//     -> everything is flatten (has a flatten after)
//     -> Everything is map, or flatten, or map+flatten.
//
// The Final Fold: is "flatten"
//
// Monotonic, non-mophisms: count on (multi)set.
//
// Want an element => (very restricted) filter.
//         then fold join.
//         What does the client do with the thing.
//         -> Depends on what kind of lattice it is :)
//         -> In general can only safely check membership and
//
// Lvars read without Top lets us preserve monotonicity. But not determinism.

pub trait Set<T> {

}


pub struct SingletonSet<T>(T);

impl <T> From<T> for SingletonSet<T> {
    fn from(val: T) -> Self {
        SingletonSet(val)
    }
}


// #[test]
// fn test() {
//     let x: Bag<usize> = ...;
//     let mapper: Pipe<usize, String> = MapPipe::new(|i| format!("Hello {}", i));
//     let y: Bag<String> = mapper.transduce(x);

//     // Special type of Bag that is stateless?
//     // Set trait has functional stream stuff, map, filter.

//     // Transform it with a morphism. Or test read.
// }


// Morpism is a trait with an input and output.
pub trait Morphism {
    type Input;
    type Output;

    fn transduce(&self, input: Self::Input) -> Self::Output;
}



// // Pipe is a morphism from Bag to Bag.
// pub trait Pipe<S: Hash + Eq, T: Hash + Eq>:
//     Morphism<Input = Bag<S>, Output = Bag<T>>
// {}
// impl <S, T, M> Pipe<S, T> for M
// where
//     S: Hash + Eq,
//     T: Hash + Eq,
//     M: Morphism<Input = Bag<S>, Output = Bag<T>>
// {}


// /// A bag (multiset).
// /// Keeps a count, does not keep extra copies.
// pub struct Bag<T: Hash + Eq> {
//     len: usize,
//     tbl: HashMap<T, usize>,
// }

// impl <T: Hash + Eq> Bag<T> {
//     pub fn new() -> Self {
//         Self {
//             len: 0,
//             tbl: Default::default(),
//         }
//     }

//     /// Gets the total size of the bag.
//     pub fn len(&self) -> usize {
//         self.len
//     }

//     /// Inserts a single value into the bag.
//     pub fn insert(&mut self, item: T) {
//         self.len += 1;
//         self.tbl.entry(item)
//             .and_modify(|count| *count += 1)
//             .or_insert(1);
//     }

//     /// Returns how many item the bag contains. Zero means it is not contained.
//     pub fn contains(&self, item: &T) -> usize {
//         self.tbl.get(item).cloned().unwrap_or(0)
//     }
// }
