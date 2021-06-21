use super::{Lattice, LatticeRepr, Bottom};

pub struct BottomLattice<L: Lattice>{
    _phantom: std::marker::PhantomData<L>
}

impl<L: Lattice> Lattice for BottomLattice<L> {}

pub struct BottomRepr<Lr: LatticeRepr> {
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr> LatticeRepr for BottomRepr<Lr> {
    type Lattice = BottomLattice<Lr::Lattice>;
    type Repr = Option<Lr::Repr>;
}

impl<Lr: LatticeRepr> Bottom for BottomRepr<Lr> {
    fn is_bottom(this: &Self::Repr) -> bool {
        this.is_none()
    }
}
