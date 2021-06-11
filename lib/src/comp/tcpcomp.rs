use std::cell::RefCell;
use std::future::Future;
use std::convert::TryInto;

use bytes::Bytes;

use tokio::io::{AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;

use crate::collections::Collection;
use crate::lattice::LatticeRepr;
use crate::lattice::setunion::{SetUnion};
use crate::op::OpDelta;

use super::{Comp, Next};

pub struct TcpComp<O: OpDelta>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<Bytes>>,
    <O::LatRepr as LatticeRepr>::Repr: Collection<Bytes, ()>,
{
    op: O,
    tcp_write: RefCell<OwnedWriteHalf>,
}

impl<O: OpDelta> TcpComp<O>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<Bytes>>,
    <O::LatRepr as LatticeRepr>::Repr: Collection<Bytes, ()>,
{
    pub fn new(op: O, tcp_write: OwnedWriteHalf) -> Self {
        Self {
            op,
            tcp_write: RefCell::new(tcp_write),
        }
    }
}

impl<O: OpDelta> Comp for TcpComp<O>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<Bytes>>,
    <O::LatRepr as LatticeRepr>::Repr: Collection<Bytes, ()>,
{
    type Error = tokio::io::Error;

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            if let Some(hide) = (Next { op: &self.op }).await {
                for bytes in hide.reveal_ref().keys() {
                    let mut tcp_write_mut = self.tcp_write.borrow_mut();
                    // TODO use the encoder.
                    let len = bytes.len().try_into().unwrap_or_else(|_| panic!("Message too long! {}", bytes.len()));
                    tcp_write_mut.write_u16(len).await?;
                    tcp_write_mut.write_all(&*bytes).await?;
                }
                Ok(())
            }
            else {
                Err(tokio::io::Error::new(std::io::ErrorKind::UnexpectedEof, "End of stream."))
            }
        }
    }
}
