use bytes::Bytes;

use super::LatticeRepr;

pub struct BytesRepr<Lr: LatticeRepr> {
    _phantom: std::marker::PhantomData<Lr>,
}
impl<Lr: LatticeRepr> LatticeRepr for BytesRepr<Lr> {
    type Lattice = Lr::Lattice;
    type Repr = Bytes;
}
