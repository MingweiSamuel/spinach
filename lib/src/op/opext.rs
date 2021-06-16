use std::net::SocketAddr;

use bytes::Bytes;

use tokio::net::tcp::OwnedWriteHalf;

use crate::collections::Collection;
use crate::comp::{DebugComp, TcpComp, TcpServerComp};
use crate::func::unary::Morphism;
use crate::lattice::LatticeRepr;
use crate::lattice::setunion::SetUnion;
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

    fn debug_comp(self, tag: &'static str) -> DebugComp<Self>
    where
        Self: OpDelta,
        <Self::LatRepr as LatticeRepr>::Repr: std::fmt::Debug,
    {
        DebugComp::new(self, tag)
    }

    fn tcp_comp(self, tcp_write: OwnedWriteHalf) -> TcpComp<Self>
    where
        Self: OpDelta,
        Self::LatRepr: LatticeRepr<Lattice = SetUnion<Bytes>>,
        <Self::LatRepr as LatticeRepr>::Repr: Collection<Bytes, ()>,
    {
        TcpComp::new(self, tcp_write)
    }

    fn tcp_server_comp(self, tcp_server: TcpServer) -> TcpServerComp<Self>
    where
        Self: OpDelta,
        Self::LatRepr: LatticeRepr<Lattice = SetUnion<(SocketAddr, Bytes)>>,
        <Self::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = (SocketAddr, Bytes)>,
    {
        TcpServerComp::new(self, tcp_server)
    }
}