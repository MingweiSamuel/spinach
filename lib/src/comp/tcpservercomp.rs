use std::future::Future;
use std::net::SocketAddr;

use bytes::Bytes;

use crate::lattice::LatticeRepr;
use crate::lattice::set_union::{SetUnion};
use crate::op::OpDelta;
use crate::tcp_server::TcpServer;

use super::{Comp, Next};

pub struct TcpServerComp<O: OpDelta>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<(SocketAddr, Bytes)>>,
    <O::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = (SocketAddr, Bytes)>,
{
    op: O,
    tcp_server: TcpServer,
}

impl<O: OpDelta> TcpServerComp<O>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<(SocketAddr, Bytes)>>,
    <O::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = (SocketAddr, Bytes)>,
{
    pub fn new(op: O, tcp_server: TcpServer) -> Self {
        Self {
            op,
            tcp_server,
        }
    }
}

impl<O: OpDelta> Comp for TcpServerComp<O>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<(SocketAddr, Bytes)>>,
    <O::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = (SocketAddr, Bytes)>,
{
    type Error = tokio::io::Error;

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            if let Some(hide) = (Next { op: &self.op }).await {
                for (addr, item) in hide.into_reveal().into_iter() {
                    self.tcp_server.write(addr, item).await?;
                }
                Ok(())
            }
            else {
                Err(tokio::io::Error::new(std::io::ErrorKind::UnexpectedEof, "End of stream."))
            }
        }
    }
}
