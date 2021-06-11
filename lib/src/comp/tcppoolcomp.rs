use std::future::Future;
use std::net::SocketAddr;

use serde::ser::Serialize;

use crate::collections::Collection;
use crate::lattice::LatticeRepr;
use crate::lattice::setunion::{SetUnion};
use crate::op::OpDelta;
use crate::tcp_pool::TcpPool;

use super::{Comp, Next};

pub struct TcpPoolComp<O: OpDelta, T: Clone + Serialize>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<(SocketAddr, T)>>,
    <O::LatRepr as LatticeRepr>::Repr: Collection<(SocketAddr, T), ()>,
{
    op: O,
    tcp_pool: TcpPool,
}

impl<O: OpDelta, T: Clone + Serialize> TcpPoolComp<O, T>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<(SocketAddr, T)>>,
    <O::LatRepr as LatticeRepr>::Repr: Collection<(SocketAddr, T), ()>,
{
    pub fn new(op: O, tcp_pool: TcpPool) -> Self {
        Self {
            op,
            tcp_pool,
        }
    }
}

impl<O: OpDelta, T: Clone + Serialize> Comp for TcpPoolComp<O, T>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<(SocketAddr, T)>>,
    <O::LatRepr as LatticeRepr>::Repr: Collection<(SocketAddr, T), ()>,
{
    type Error = tokio::io::Error;

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            if let Some(hide) = (Next { op: &self.op }).await {
                for (addr, item) in hide.reveal_ref().keys() {
                    self.tcp_pool.write(*addr, item).await?;
                }
                Ok(())
            }
            else {
                Err(tokio::io::Error::new(std::io::ErrorKind::UnexpectedEof, "End of stream."))
            }
        }
    }
}
