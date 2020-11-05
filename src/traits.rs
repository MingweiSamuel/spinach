use std::iter::FromIterator;

use crate::merge::Merge;
use crate::semilattice::Semilattice;

pub trait UnaryFn {
    type Domain;
    type Codomain;

    fn call(input: Self::Domain) -> Self::Codomain;
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


// impl <T> Set for Option<T> {
//     type Domain = T;

//     fn map<F, C>(self) -> C
//     where
//         F: UnaryFn<Domain = T>,
//         C: FromIterator<F::Codomain> + Set
//     {
//         self.into_iter()
//             .map(|x| F::call(x))
//             .collect()
//     }
// }

// impl <T> Set for BTreeSet<T> {
//     type Domain = T;

//     fn map<F, C>(self) -> C
//     where
//         F: UnaryFn<Domain = T>,
//         C: FromIterator<F::Codomain> + Set,
//     {
//         self.into_iter()
//             .map(|x| F::call(x))
//             .collect()
//     }
// }

// impl <T> Set for HashSet<T> {
//     type Domain = T;

//     fn map<F, C>(self) -> C
//     where
//         F: UnaryFn<Domain = T>,
//         C: FromIterator<F::Codomain> + Set,
//     {
//         self.into_iter()
//             .map(|x| F::call(x))
//             .collect()
//     }
// }
