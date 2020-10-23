//! 1. A UnaryFunction is a function from a domain to a codomain.
//!    - It should be deterministic (ProcMacro here).
//! 2. A SemilatticeHomomorphism is a function from a semilattice to a (possibly different) semilattice.
//!    - It should be a morphism (ProcMacro here).
//! 3. A MapFunction. It is a SemilatticeHomomorphism from Lattice<BTreeSet<T>, UnionMerge> to Lattice<BTreeSet<U>, UnionMerge>.
//!    Constructor takes in a UnaryFunction and turns it into a MapFunction which is a SemilatticeHomomorphism.
//! 4. A MergeFoldFunction. It is a SemilatticeHomomorphism which takes in a Lattice<BTreeSet<Lattice<T, M>>, UnionMerge> and returns
//!    and returns a single Lattice<T, M> by folding over the merge function M.
//!    There is no constructor.

use std::collections::{ BTreeSet, HashMap };

use crate::lattice::Semilattice;
use crate::merge::{ Merge, UnionMerge, MapUnionMerge };

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
// // All `SemilatticeHomomorphism`s are `UnaryFunction`s.
// impl <F: SemilatticeHomomorphism> UnaryFunction for F {
//     type Domain = Semilattice<
//         <Self as SemilatticeHomomorphism>::DomainCarrier,
//         <Self as SemilatticeHomomorphism>::DomainMerge>;
//     type Codomain = Semilattice<
//         <Self as SemilatticeHomomorphism>::CodomainCarrier,
//         <Self as SemilatticeHomomorphism>::CodomainMerge>;

//     fn call(input: Self::Domain) -> Self::Codomain {
//         F::call(input)
//     }
// }

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
    <F as UnaryFunction>::Domain:   Eq + Ord,
    <F as UnaryFunction>::Codomain: Eq + Ord,
{
    type DomainCarrier = BTreeSet<F::Domain>;
    type DomainMerge = UnionMerge;
    type CodomainCarrier = BTreeSet<F::Codomain>;
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
/// A SemilatticeHomomorphism from `Lattice<BTreeSet<Lattice<I, M>>>` to
/// `Lattice<I, M>` by folding using the `M` merge function.
pub struct MergeFoldFunction<DomainCarrier, DomainMerge>
where
    DomainCarrier: Default, // Needs a bound. (TODO)
    DomainMerge: Merge<DomainCarrier>,
    Semilattice<DomainCarrier, DomainMerge>: Eq + Ord,
{
    _phantom: std::marker::PhantomData<( DomainCarrier, DomainMerge )>,
}
impl <CD, CDM> SemilatticeHomomorphism for MergeFoldFunction<CD, CDM>
where
    CD: Default,
    CDM: Merge<CD>,
    Semilattice<CD, CDM>: Eq + Ord,
{
    type DomainCarrier = BTreeSet<Semilattice<CD, CDM>>;
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

#[cfg(test)]
mod test {
    use super::*;

    type FirstLastName = (&'static str, &'static str);

    #[test]
    fn test_select() {
        struct SelectMNames;
        impl UnaryFunction for SelectMNames {
            type Domain = FirstLastName;
            type Codomain = Semilattice<BTreeSet<FirstLastName>, UnionMerge>;

            fn call(input: Self::Domain) -> Self::Codomain {
                let mut out = BTreeSet::new();
                if input.0.starts_with('M') {
                    out.insert(input);
                }
                out.into() // Convert to Semilattice<.,.>.
            }
        }

        let x0: Semilattice<BTreeSet<FirstLastName>, UnionMerge> = vec![
            ( "Joseph", "Hellerstein" ),
            ( "Matthew", "Milano" ),
            ( "Mingwei", "Samuel" ),
            ( "Pranav", "Gaddamadugu" ),
        ].into_iter().collect::<BTreeSet<_>>().into();

        let x1: Semilattice<
            BTreeSet<Semilattice<BTreeSet<FirstLastName>, UnionMerge>>,
            UnionMerge
        > = MapFunction::<SelectMNames>::call(x0);

        let x2: Semilattice<BTreeSet<FirstLastName>, UnionMerge> =
            MergeFoldFunction::call(x1);

        assert_eq!(
            vec![
                ( "Matthew", "Milano" ),
                ( "Mingwei", "Samuel" ),
            ].into_iter().collect::<BTreeSet<_>>(),
            x2.into_reveal());
    }

    #[test]
    fn test_project() {
        struct ProjectFirst;
        impl UnaryFunction for ProjectFirst {
            type Domain = FirstLastName;
            type Codomain = &'static str;

            fn call(input: Self::Domain) -> Self::Codomain {
                input.0
            }
        }

        let x0: Semilattice<BTreeSet<FirstLastName>, UnionMerge> = vec![
            ( "Joseph", "Hellerstein" ),
            ( "Matthew", "Milano" ),
            ( "Mingwei", "Samuel" ),
            ( "Pranav", "Gaddamadugu" ),
        ].into_iter().collect::<BTreeSet<_>>().into();

        let x1: Semilattice<BTreeSet<&'static str>, UnionMerge> =
            MapFunction::<ProjectFirst>::call(x0);

        assert_eq!(
            vec![
                "Joseph",
                "Matthew",
                "Mingwei",
                "Pranav",
            ].into_iter().collect::<BTreeSet<_>>(),
            x1.into_reveal());
    }

    // #[test]
    // fn test_groupby() {
    //     struct GroupByNameLength;
    //     impl UnaryFunction for GroupByNameLength {
    //         type Domain = FirstLastName;
    //         type Codomain = Semilattice<
    //             HashMap<usize, Semilattice<BTreeSet<FirstLastName>, UnionMerge>>,
    //             MapUnionMerge>;

    //         fn call(input: Self::Domain) -> Self::Codomain {
    //             let set = BTreeSet::new();
    //             set.insert(input);

    //             let out = HashMap::new();
    //             out.insert(input.0.len(), set);
    //             out
    //         }
    //     }

    //     let x0: Semilattice<BTreeSet<FirstLastName>, UnionMerge> = vec![
    //         ( "Joseph", "Hellerstein" ),
    //         ( "Matthew", "Milano" ),
    //         ( "Mingwei", "Samuel" ),
    //         ( "Pranav", "Gaddamadugu" ),
    //     ].into_iter().collect::<BTreeSet<_>>().into();

    //     let x1 = MapFunction::<GroupByNameLength>::call(x0);

    //     let x2: Semilattice<
    //         HashMap<usize, Semilattice<BTreeSet<FirstLastName>, UnionMerge>>,
    //         MapUnionMerge
    //     > = MergeFoldFunction::call(x1);

    //     println!("{}", x2.into_reveal());
    //     // assert_eq!(
    //     //     vec![
    //     //         "Joseph",
    //     //         "Matthew",
    //     //         "Mingwei",
    //     //         "Pranav",
    //     //     ].into_iter().collect::<BTreeSet<_>>(),
    //     //     x1.into_reveal());
    // }
}


// TODO: use https://crates.io/crates/derivative