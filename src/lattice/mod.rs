use crate::hide::{Hide, Type};

pub mod setunion;
pub mod mapunion;
pub mod ord;
pub mod dompair;

pub trait Lattice {}

pub trait LatticeRepr {
    type Lattice: Lattice;
    type Repr: Clone;
}

pub trait Merge<Delta: LatticeRepr>: LatticeRepr<Lattice = Delta::Lattice> {
    /// Merge DELTA into THIS. Return TRUE if THIS changed, FALSE if THIS was unchanged.
    fn merge(this: &mut Self::Repr, delta: Delta::Repr) -> bool;

    fn merge_hide<Y: Type, Z: Type>(this: &mut Hide<Y, Self>, delta: Hide<Z, Delta>) -> bool {
        Self::merge(this.as_reveal_mut(), delta.into_reveal())
    }
}

pub trait Convert<Target: LatticeRepr<Lattice = Self::Lattice>>: LatticeRepr {
    fn convert(this: Self::Repr) -> Target::Repr;

    fn convert_hide<Y: Type>(this: Hide<Y, Self>) -> Hide<Y, Target> {
        Hide::new(Self::convert(this.into_reveal()))
    }
}

pub trait Compare<Other: LatticeRepr<Lattice = Self::Lattice>>: LatticeRepr {
    fn compare(this: &Self::Repr, other: &Other::Repr) -> Option<std::cmp::Ordering>;
}
