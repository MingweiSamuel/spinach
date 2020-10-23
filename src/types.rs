//! 1. A UnaryFunction is a function from a domain to a codomain.
//!    - It should be deterministic (ProcMacro here).
//! 2. A SemilatticeHomomorphism is a function from a semilattice to a (possibly different) semilattice.
//!    - It should be a morphism (ProcMacro here).
//! 3. A MapFunction. It is a SemilatticeHomomorphism from Lattice<HashSet<T>, UnionMerge> to Lattice<HashSet<U>, UnionMerge>.
//!    Constructor takes in a UnaryFunction and turns it into a MapFunction which is a SemilatticeHomomorphism.
//! 4. A MergeFoldFunction. It is a SemilatticeHomomorphism which takes in a Lattice<HashSet<Lattice<T, M>>, UnionMerge> and returns
//!    and returns a single Lattice<T, M> by folding over the merge function M.
//!    There is no constructor.

use std::collections::HashSet;
use std::hash::Hash;

use crate::lattice::Semilattice;
use crate::merge::{ Merge, UnionMerge };

/// Must be deterministic!!!
pub trait UnaryFunction {
    type Domain;
    type Codomain;

    fn call(input: Self::Domain) -> Self::Codomain;
}

/// Must distribute over merge!!! ("structure preserving")
pub trait SemilatticeHomomorphism {
    type DomainCarrier;
    type DomainMerge: Merge<Self::DomainCarrier>;
    type CodomainCarrier;
    type CodomainMerge: Merge<Self::CodomainCarrier>;

    fn call(input: Semilattice<Self::DomainCarrier, Self::DomainMerge>)
        -> Semilattice<Self::CodomainCarrier, Self::CodomainMerge>;
}
// All `SemilatticeHomomorphism`s are `UnaryFunction`s.
impl <F: SemilatticeHomomorphism> UnaryFunction for F {
    type Domain = Semilattice<
        <Self as SemilatticeHomomorphism>::DomainCarrier,
        <Self as SemilatticeHomomorphism>::DomainMerge>;
    type Codomain = Semilattice<
        <Self as SemilatticeHomomorphism>::CodomainCarrier,
        <Self as SemilatticeHomomorphism>::CodomainMerge>;

    fn call(input: Self::Domain) -> Self::Codomain {
        F::call(input)
    }
}

/// Takes in a `UnaryFunction` and gives back a MapFunction which is a `SemilatticeHomomorphism`.
pub struct MapFunction<F: UnaryFunction> {
    _phantom: std::marker::PhantomData<F>,
}
impl <F: UnaryFunction> MapFunction<F> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl <F: UnaryFunction> SemilatticeHomomorphism for MapFunction<F>
where
    <F as UnaryFunction>::Domain:   Eq + Hash,
    <F as UnaryFunction>::Codomain: Eq + Hash,
{
    type DomainCarrier = HashSet<F::Domain>;
    type DomainMerge = UnionMerge;
    type CodomainCarrier = HashSet<F::Codomain>;
    type CodomainMerge = UnionMerge;

    fn call(input: Semilattice<Self::DomainCarrier, Self::DomainMerge>)
        -> Semilattice<Self::CodomainCarrier, Self::CodomainMerge>
    {
        input.into_reveal() // REVEAL HERE!
            .into_iter()
            .map(|x| F::call(x))
            .collect::<Self::CodomainCarrier>()
            .into()
    }
}

/// `MergeFoldFunction` is a general fold function via semilattice merge.
/// A SemilatticeHomomorphism from `Lattice<HashSet<Lattice<I, M>>>` to
/// `Lattice<I, M>` by folding using the `M` merge function.
pub struct MergeFoldFunction<DomainCarrier, DomainMerge>
where
    DomainCarrier: Default, // Needs a bound. (TODO)
    DomainMerge: Merge<DomainCarrier>,
    Semilattice<DomainCarrier, DomainMerge>: Hash + Eq,
{
    _phantom: std::marker::PhantomData<( DomainCarrier, DomainMerge )>,
}
impl <CD, CDM> SemilatticeHomomorphism for MergeFoldFunction<CD, CDM>
where
    CD: Default,
    CDM: Merge<CD>,
    Semilattice<CD, CDM>: Hash + Eq,
{
    type DomainCarrier = HashSet<Semilattice<CD, CDM>>;
    type DomainMerge = UnionMerge;
    type CodomainCarrier = CD;
    type CodomainMerge = CDM;

    fn call(input: Semilattice<Self::DomainCarrier, Self::DomainMerge>)
        -> Semilattice<Self::CodomainCarrier, Self::CodomainMerge>
    {
        input.into_reveal() // REVEAL HERE!
            .into_iter()
            .fold(CD::default(), |mut acc, x| {
                CDM::merge(&mut acc, x.into_reveal()); // REVEAL HERE!
                acc
            })
            .into()
    }
}


// TODO: use https://crates.io/crates/derivative


// Lattice -
// Morphism
//



// pub trait Set<T> {

// }


// pub struct SingletonSet<T>(T);

// impl <T> From<T> for SingletonSet<T> {
//     fn from(val: T) -> Self {
//         SingletonSet(val)
//     }
// }


// #[test]
// fn test() {
//     let x: Bag<usize> = ...;
//     let mapper: Pipe<usize, String> = MapPipe::new(|i| format!("Hello {}", i));
//     let y: Bag<String> = mapper.transduce(x);

//     // Special type of Bag that is stateless?
//     // Set trait has functional stream stuff, map, filter.

//     // Transform it with a morphism. Or test read.
// }


// // Morpism is a trait with an input and output.
// pub trait Morphism {
//     type Input;
//     type Output;

//     fn transduce(&self, input: Self::Input) -> Self::Output;
// }



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
