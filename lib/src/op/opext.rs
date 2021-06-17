use std::net::SocketAddr;

use bytes::Bytes;

use tokio::net::tcp::OwnedWriteHalf;

use crate::collections::Collection;
use crate::comp::{DebugComp, NullComp, TcpComp, TcpServerComp};
use crate::func::unary::Morphism;
use crate::func::binary::BinaryMorphism;
use crate::lattice::{Convert, LatticeRepr, Merge};
use crate::lattice::setunion::SetUnion;
use crate::lattice::pair::PairRepr;
use crate::tcp_server::TcpServer;

use super::*;

impl<O> OpExt for O where O: Op {}

pub trait OpExt: Sized + Op {
    fn debug(self, tag: &'static str) -> DebugOp<Self>
    where
        <Self::LatRepr as LatticeRepr>::Repr: std::fmt::Debug,
    {
        DebugOp::new(self, tag)
    }

    fn morphism<F: Morphism<InLatRepr = Self::LatRepr>>(self, func: F) -> MorphismOp<Self, F> {
        MorphismOp::new(self, func)
    }

    fn lattice<Lr: LatticeRepr + Merge<Self::LatRepr>>(self, bottom: Lr::Repr) -> LatticeOp<Self, Lr>
    where
        Self::LatRepr: Convert<Lr>,
    {
        LatticeOp::new(self, bottom)
    }

    fn binary<F, O: OpValue>(self, op: O, func: F) -> BinaryOp<Self, O, F>
    where
        Self: OpValue,
        F: BinaryMorphism<InLatReprA = Self::LatRepr, InLatReprB = O::LatRepr>,
    {
        BinaryOp::new(self, op, func)
    }

    fn lattice_default<Lr: LatticeRepr + Merge<Self::LatRepr>>(self) -> LatticeOp<Self, Lr>
    where
        Self::LatRepr: Convert<Lr>,
        Lr::Repr: Default,
    {
        LatticeOp::new_default(self)
    }

    fn fixed_split<const N: usize>(self) -> [SplitOp<Self>; N] {
        fixed_split(self)
    }

    fn dyn_split(self) -> Splitter<Self>
    where
        Self: OpValue,
    {
        Splitter::new(self)
    }

    fn switch<Ra: LatticeRepr, Rb: LatticeRepr>(self) -> (SwitchOp<Self, Ra, Rb, switch::SwitchModeA>, SwitchOp<Self, Ra, Rb, switch::SwitchModeB>)
    where
        Self: Op<LatRepr = PairRepr<Ra, Rb>>,
    {
        SwitchOp::new(self)
    }

    fn comp_debug(self, tag: &'static str) -> DebugComp<Self>
    where
        Self: OpDelta,
        <Self::LatRepr as LatticeRepr>::Repr: std::fmt::Debug,
    {
        DebugComp::new(self, tag)
    }

    fn comp_null(self) -> NullComp<Self>
    where
        Self: OpDelta,
    {
        NullComp::new(self)
    }

    fn comp_tcp(self, tcp_write: OwnedWriteHalf) -> TcpComp<Self>
    where
        Self: OpDelta,
        Self::LatRepr: LatticeRepr<Lattice = SetUnion<Bytes>>,
        <Self::LatRepr as LatticeRepr>::Repr: Collection<Bytes, ()>,
    {
        TcpComp::new(self, tcp_write)
    }

    fn comp_tcp_server(self, tcp_server: TcpServer) -> TcpServerComp<Self>
    where
        Self: OpDelta,
        Self::LatRepr: LatticeRepr<Lattice = SetUnion<(SocketAddr, Bytes)>>,
        <Self::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = (SocketAddr, Bytes)>,
    {
        TcpServerComp::new(self, tcp_server)
    }
}