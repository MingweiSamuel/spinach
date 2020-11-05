use std::iter::FromIterator;

use crate::merge::Merge;
use crate::semilattice::Semilattice;



pub trait Set {
    type Domain;

    fn map_into<F, C>(self) -> C
    where
        F: UnaryFn<Domain = Self::Domain>,
        C: FromIterator<F::Codomain> + Set;

    fn fold_into<F>(self, target: &mut Semilattice<F>)
    where
        F: Merge<Domain = Self::Domain>;
}


pub struct SetUnion<X> {
    _phantom: std::marker::PhantomData<X>,
}
impl <X> Merge for SetUnion<X>
where
    X: Set + Extend<<X as Set>::Domain> + IntoIterator<Item = <X as Set>::Domain>,
{
    type Domain = X;

    fn merge(val: &mut X, other: X) {
        val.extend(other.into_iter());
    }

    fn partial_cmp(_val: &X, _other: &X) -> Option<std::cmp::Ordering> {
        todo!("Not implemented!");
    }
}


pub trait UnaryFn {
    type Domain;
    type Codomain;

    fn call(input: Self::Domain) -> Self::Codomain;
}


pub trait SemilatticeMorphismFn {
    type DomainMerge: Merge;
    type CodomainMerge: Merge;

    fn call(input: Semilattice<Self::DomainMerge>) -> Semilattice<Self::CodomainMerge>;
}


pub struct MapMorphismFn<F: UnaryFn, D, C> {
    _phantom: std::marker::PhantomData<(F, D, C)>,
}
impl <F: UnaryFn, D, C> SemilatticeMorphismFn for MapMorphismFn<F, D, C>
where
    D: Set<Domain = F::Domain> + Extend<F::Domain> + IntoIterator<Item = F::Domain>,
    C: Set<Domain = F::Codomain> + Extend<F::Codomain> + IntoIterator<Item = F::Codomain> + FromIterator<F::Codomain>,
{
    type DomainMerge = SetUnion<D>;
    type CodomainMerge = SetUnion<C>;

    fn call(input: Semilattice<Self::DomainMerge>) -> Semilattice<Self::CodomainMerge> {
        let val = input.into_reveal() // REVEAL HERE!
            .map_into::<F, C>();
        Semilattice::new(val)
    }
}


pub struct FoldMorphismFn<D, F: Merge>
where
    D: Set<Domain = F::Domain> + IntoIterator<Item = F::Domain> + Extend<F::Domain>,
    Semilattice<F>: Default,
{
    _phantom: std::marker::PhantomData<(D, F)>,
}
impl <D, F: Merge> SemilatticeMorphismFn for FoldMorphismFn<D, F>
where
    D: Set<Domain = F::Domain> + IntoIterator<Item = F::Domain> + Extend<F::Domain>,
    Semilattice<F>: Default,
{
    type DomainMerge = SetUnion<D>;
    type CodomainMerge = F;

    fn call(input: Semilattice<Self::DomainMerge>) -> Semilattice<Self::CodomainMerge> {
        let mut val = Semilattice::default();
        input.into_reveal() // REVEAL HERE!
            .fold_into::<F>(&mut val);
        val
    }
}


#[cfg(test)]
mod test {
    use super::*;

    use std::collections::{ BTreeSet };

    type FirstLastName = (&'static str, &'static str);

    #[test]
    fn test_identity() {
        struct IdentityUnaryFn<T> {
            _phantom: std::marker::PhantomData<T>,
        }
        impl <T> UnaryFn for IdentityUnaryFn<T> {
            type Domain = T;
            type Codomain = T;

            fn call(input: T) -> T {
                input
            }
        }

        type IdentityMorphismFn = MapMorphismFn::<IdentityUnaryFn<FirstLastName>, BTreeSet<FirstLastName>, BTreeSet<FirstLastName>>;

        let x0: Semilattice<SetUnion<BTreeSet<FirstLastName>>> = Semilattice::new(vec![
            ( "Joseph", "Hellerstein" ),
            ( "Matthew", "Milano" ),
            ( "Mingwei", "Samuel" ),
            ( "Pranav", "Gaddamadugu" ),
        ].into_iter().collect::<BTreeSet<_>>());

        let x1: Semilattice<SetUnion<BTreeSet<FirstLastName>>> = IdentityMorphismFn::call(x0);

        println!("Identity: {:?}", x1.into_reveal());
    }

    #[test]
    fn test_filter() {
        struct MFilterFn;
        impl UnaryFn for MFilterFn {
            type Domain = FirstLastName;
            type Codomain = BTreeSet<FirstLastName>;

            fn call(input: FirstLastName) -> BTreeSet<FirstLastName> {
                let mut val = BTreeSet::new();
                if input.0.starts_with('M') {
                    val.insert(input);
                }
                val
            }
        }

        type MFilterMorphismFn = MapMorphismFn::<MFilterFn, BTreeSet<FirstLastName>, BTreeSet<BTreeSet<FirstLastName>>>;

        type FoldFilterMorphismFn = FoldMorphismFn::<BTreeSet<BTreeSet<FirstLastName>>, SetUnion<BTreeSet<FirstLastName>>>;

        let x0: Semilattice<SetUnion<BTreeSet<FirstLastName>>> = Semilattice::new(vec![
            ( "Joseph", "Hellerstein" ),
            ( "Matthew", "Milano" ),
            ( "Mingwei", "Samuel" ),
            ( "Pranav", "Gaddamadugu" ),
        ].into_iter().collect::<BTreeSet<_>>());

        let x1: Semilattice<SetUnion<BTreeSet<BTreeSet<FirstLastName>>>> = MFilterMorphismFn::call(x0);

        let x2: Semilattice<SetUnion<BTreeSet<FirstLastName>>> = FoldFilterMorphismFn::call(x1);

        println!("Filter: {:?}", x2.into_reveal());
    }
}














impl <T, X> Set for X
where
    X: IntoIterator<Item = T>
{
    type Domain = T;

    fn map_into<F, C>(self) -> C
    where
        F: UnaryFn<Domain = Self::Domain>,
        C: FromIterator<F::Codomain> + Set,
    {
        self.into_iter()
            .map(|x| F::call(x))
            .collect()
    }

    fn fold_into<F>(self, target: &mut Semilattice<F>)
    where
        F: Merge<Domain = Self::Domain>,
    {
        for x in self {
            target.merge_in(x);
        }
    }
}
