pub mod setunion;
pub mod mapunion;

pub trait Lattice {}

pub trait LatticeRepr {
    type Lattice: Lattice;
    type Repr;
}

pub trait Merge<Delta: LatticeRepr>: LatticeRepr<Lattice = Delta::Lattice> {
    fn merge(this: &mut Self::Repr, delta: Delta::Repr);
}

pub trait Convert<Target: LatticeRepr<Lattice = Self::Lattice>>: LatticeRepr {
    fn convert(this: Self::Repr) -> Target::Repr;
}
