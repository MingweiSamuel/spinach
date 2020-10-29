//! 1. A UnaryFunction is a function from a domain to a codomain.
//!    - It should be deterministic (ProcMacro here).
//! 2. A SemilatticeHomomorphism is a function from a semilattice to a (possibly different) semilattice.
//!    - It should be a morphism (ProcMacro here).
//! 3. A MapFunction. It is a SemilatticeHomomorphism from Lattice<BTreeSet<T>, UnionMerge> to Lattice<BTreeSet<U>, UnionMerge>.
//!    Constructor takes in a UnaryFunction and turns it into a MapFunction which is a SemilatticeHomomorphism.
//! 4. A MergeFoldFunction. It is a SemilatticeHomomorphism which takes in a Lattice<BTreeSet<T>, UnionMerge> and a M: Merge<T>
//!    and returns a single Lattice<T, M> by folding over M.
//!    There is no constructor.

use std::collections::{ BTreeSet };

use crate::lattice::Semilattice;
use crate::merge::{ Merge, UnionMerge };

/// Must be deterministic!!!
/// And probably just be a pure function in general?!
pub trait UnaryFunction {
    type Domain;
    type Codomain;

    fn call(input: Self::Domain) -> Self::Codomain;
}

/// Must distribute over merge!!! ("structure preserving")
/// And see above!
pub trait SemilatticeHomomorphism {
    type DomainMerge: Merge;
    type CodomainMerge: Merge;

    fn call(input: Semilattice<Self::DomainMerge>) -> Semilattice<Self::CodomainMerge>;
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
impl <F: UnaryFunction> SemilatticeHomomorphism for MapFunction<F>
where
    <F as UnaryFunction>::Domain:   Eq + Ord,
    <F as UnaryFunction>::Codomain: Eq + Ord,
{
    type DomainMerge   = UnionMerge<BTreeSet<<F as UnaryFunction>::Domain>>;
    type CodomainMerge = UnionMerge<BTreeSet<<F as UnaryFunction>::Codomain>>;

    fn call(input: Semilattice<Self::DomainMerge>) -> Semilattice<Self::CodomainMerge>
    {
        input.map_into::<F>()
        // let val = input.into_reveal() // REVEAL HERE!
        //     .into_iter()
        //     .map(|x| F::call(x))
        //     .collect::<BTreeSet<<F as UnaryFunction>::Codomain>>();
        // Semilattice::new(val)
    }
}

/// `MergeFoldFunction` is a general fold function via semilattice merge.
/// A SemilatticeHomomorphism from `Lattice<BTreeSet<Lattice<I, M>>>` to
/// `Lattice<I, M>` by folding using the `M` merge function.
pub struct MergeFoldFunction<DomainMerge>
where
    DomainMerge: Merge,
    <DomainMerge as Merge>::Domain: Default, // Needs a bound. (TODO)
{
    _phantom: std::marker::PhantomData<DomainMerge>,
}

impl <DM> SemilatticeHomomorphism for MergeFoldFunction<DM>
where
    DM: Merge,
    <DM as Merge>::Domain: Default + Ord + Eq,
{
    type DomainMerge = UnionMerge<BTreeSet<<DM as Merge>::Domain>>;
    type CodomainMerge = DM;

    fn call(input: Semilattice<Self::DomainMerge>) -> Semilattice<Self::CodomainMerge>
    {
        let val = input.into_reveal() // REVEAL HERE!
            .into_iter()
            .fold(<DM as Merge>::Domain::default(), |mut acc, x| {
                DM::merge(&mut acc, x);
                acc
            });
        Semilattice::new(val)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::collections::{ BTreeMap };

    use crate::merge::{ MapUnionMerge };

    type FirstLastName = (&'static str, &'static str);

    #[test]
    fn test_identity() {
        struct IdentityUnaryFunction<T> {
            _phantom: std::marker::PhantomData<T>,
        }
        impl <T> UnaryFunction for IdentityUnaryFunction<T> {
            type Domain = T;
            type Codomain = T;

            fn call(input: T) -> T {
                input
            }
        }

        type IdentityMorphism = MapFunction::
            <IdentityUnaryFunction<(&'static str, &'static str)>>;


        let x0: Semilattice<UnionMerge<BTreeSet<FirstLastName>>> = Semilattice::new(vec![
            ( "Joseph", "Hellerstein" ),
            ( "Matthew", "Milano" ),
            ( "Mingwei", "Samuel" ),
            ( "Pranav", "Gaddamadugu" ),
        ].into_iter().collect::<BTreeSet<_>>());

        let x1: Semilattice<UnionMerge<BTreeSet<FirstLastName>>> = IdentityMorphism::call(x0);

        println!("{:?}", x1.into_reveal());
    }

    #[test]
    fn test_select() {
        struct SelectMNames;
        impl UnaryFunction for SelectMNames {
            type Domain = FirstLastName;
            type Codomain = BTreeSet<FirstLastName>;

            fn call(input: Self::Domain) -> Self::Codomain {
                let mut out = BTreeSet::new();
                if input.0.starts_with('M') {
                    out.insert(input);
                }
                out
            }
        }

        let x0: Semilattice<UnionMerge<BTreeSet<FirstLastName>>> = Semilattice::new(vec![
            ( "Joseph", "Hellerstein" ),
            ( "Matthew", "Milano" ),
            ( "Mingwei", "Samuel" ),
            ( "Pranav", "Gaddamadugu" ),
        ].into_iter().collect::<BTreeSet<_>>());

        let x1: Semilattice<
            UnionMerge<BTreeSet<BTreeSet<FirstLastName>>>,
        > = MapFunction::<SelectMNames>::call(x0);

        let x2: Semilattice<UnionMerge<BTreeSet<FirstLastName>>> =
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

        let x0: Semilattice<UnionMerge<BTreeSet<FirstLastName>>> = Semilattice::new(vec![
            ( "Joseph", "Hellerstein" ),
            ( "Matthew", "Milano" ),
            ( "Mingwei", "Samuel" ),
            ( "Pranav", "Gaddamadugu" ),
        ].into_iter().collect::<BTreeSet<_>>().into());

        let x1: Semilattice<UnionMerge<BTreeSet<&'static str>>> =
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

    #[test]
    fn test_groupby() {
        struct GroupByNameLength;
        impl UnaryFunction for GroupByNameLength {
            type Domain = FirstLastName;
            type Codomain = BTreeMap<usize, BTreeSet<FirstLastName>>;

            fn call(input: Self::Domain) -> Self::Codomain {
                let mut set = BTreeSet::new();
                set.insert(input);

                let mut out = BTreeMap::new();
                out.insert(input.0.len(), set);
                out
            }
        }

        let x0: Semilattice<UnionMerge<BTreeSet<FirstLastName>>> = Semilattice::new(vec![
            ( "Joseph", "Hellerstein" ),
            ( "Matthew", "Milano" ),
            ( "Mingwei", "Samuel" ),
            ( "Pranav", "Gaddamadugu" ),
        ].into_iter().collect::<BTreeSet<_>>());

        let x1 = MapFunction::<GroupByNameLength>::call(x0);

        let x2: Semilattice<
            MapUnionMerge<BTreeMap<usize, UnionMerge<BTreeSet<FirstLastName>>>>
        > = MergeFoldFunction::call(x1);

        println!("{:#?}", x2.into_reveal());
        // {
        //     6: {
        //         (
        //             "Joseph",
        //             "Hellerstein",
        //         ),
        //         (
        //             "Pranav",
        //             "Gaddamadugu",
        //         ),
        //     },
        //     7: {
        //         (
        //             "Matthew",
        //             "Milano",
        //         ),
        //         (
        //             "Mingwei",
        //             "Samuel",
        //         ),
        //     },
        // }
    }

    // #[test]
    // fn test_projectdict() {
    //     struct GroupByNameLength;
    //     impl UnaryFunction for GroupByNameLength {
    //         type Domain = FirstLastName;
    //         type Codomain = BTreeMap<usize, BTreeSet<FirstLastName>>;

    //         fn call(input: Self::Domain) -> Self::Codomain {
    //             let mut set = BTreeSet::new();
    //             set.insert(input);

    //             let mut out = BTreeMap::new();
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
    //         BTreeMap<usize, BTreeSet<FirstLastName>>,
    //         MapUnionMerge<_, UnionMerge>> = MergeFoldFunction::call(x1);

    //     let dict = x2.into_reveal();
    //     println!("{:#?}", dict);

    //     let x3: Semilattice<
    //         BTreeMap<usize, BTreeSet<FirstLastName>>,
    //         MapUnionMerge<_, UnionMerge>> = dict.into();
    // }
}
