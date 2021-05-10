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
    fn merge(this: &mut Self::Repr, delta: Delta::Repr);
}

pub trait Convert<Target: LatticeRepr<Lattice = Self::Lattice>>: LatticeRepr {
    fn convert(this: Self::Repr) -> Target::Repr;
}

pub trait Compare<Other: LatticeRepr<Lattice = Self::Lattice>>: LatticeRepr {
    fn compare(this: &Self::Repr, other: &Other::Repr) -> Option<std::cmp::Ordering>;
}