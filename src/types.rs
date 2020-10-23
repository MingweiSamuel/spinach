//! 1. A UnaryFunction is a function from a domain to a codomain.
//!    - It should be deterministic (ProcMacro here).
//! 2. A SemilatticeHomomorphism is a function from a semilattice to a (possibly different) semilattice.
//!    - It should be a morphism (ProcMacro here).
//! 3. A MapFunction. Constructor takes in a UnaryFunction and it becomes a SemilatticeHomomorphism.
//! 4. A MergeFoldFunction. It is a SemilatticeHomomorphism which takes in a HashSet<Lattice<I, M>
//!    and returns a Lattice<I, M> by folding over the merge function M.

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
    type CarrierDomain;
    type CarrierDomainMerge: Merge<Self::CarrierDomain>;
    type CarrierCodomain;
    type CarrierCodomainMerge: Merge<Self::CarrierCodomain>;

    fn call(input: Semilattice<Self::CarrierDomain, Self::CarrierDomainMerge>)
        -> Semilattice<Self::CarrierCodomain, Self::CarrierCodomainMerge>;
}
// All `SemilatticeHomomorphism`s are `UnaryFunction`s.
impl <F: SemilatticeHomomorphism> UnaryFunction for F {
    type Domain = Semilattice<
        <Self as SemilatticeHomomorphism>::CarrierDomain,
        <Self as SemilatticeHomomorphism>::CarrierDomainMerge>;
    type Codomain = Semilattice<
        <Self as SemilatticeHomomorphism>::CarrierCodomain,
        <Self as SemilatticeHomomorphism>::CarrierCodomainMerge>;

    fn call(input: Self::Domain) -> Self::Codomain {
        F::call(input)
    }
}

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
    type CarrierDomain = HashSet<F::Domain>;
    type CarrierDomainMerge = UnionMerge;
    type CarrierCodomain = HashSet<F::Codomain>;
    type CarrierCodomainMerge = UnionMerge;

    fn call(input: Semilattice<Self::CarrierDomain, Self::CarrierDomainMerge>)
        -> Semilattice<Self::CarrierCodomain, Self::CarrierCodomainMerge>
    {
        input.into_reveal() // REVEAL HERE!
            .into_iter()
            .map(|x| F::call(x))
            .collect::<Self::CarrierCodomain>()
            .into()
    }
}

pub struct MergeFoldFunction<CarrierDomain, CarrierDomainMerge>
where
    CarrierDomain: Default, // Needs a bound. (TODO)
    CarrierDomainMerge: Merge<CarrierDomain>,
    Semilattice<CarrierDomain, CarrierDomainMerge>: Hash + Eq,
{
    _phantom: std::marker::PhantomData<( CarrierDomain, CarrierDomainMerge )>,
}
impl <CD, CDM> SemilatticeHomomorphism for MergeFoldFunction<CD, CDM>
where
    CD: Default,
    CDM: Merge<CD>,
    Semilattice<CD, CDM>: Hash + Eq,
{
    type CarrierDomain = HashSet<Semilattice<CD, CDM>>;
    type CarrierDomainMerge = UnionMerge;
    type CarrierCodomain = CD;
    type CarrierCodomainMerge = CDM;

    fn call(input: Semilattice<Self::CarrierDomain, Self::CarrierDomainMerge>)
        -> Semilattice<Self::CarrierCodomain, Self::CarrierCodomainMerge>
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
