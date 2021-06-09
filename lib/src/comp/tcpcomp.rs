use std::cell::RefCell;
use std::future::Future;
use std::convert::TryInto;

use serde::ser::Serialize;
use tokio::io::{AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;

use crate::collections::Collection;
use crate::lattice::LatticeRepr;
use crate::lattice::setunion::{SetUnion};
use crate::op::OpDelta;

use super::{Comp, Next};

pub struct TcpComp<O: OpDelta, T: Clone + Serialize>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<T>>,
    <O::LatRepr as LatticeRepr>::Repr: Collection<T, ()>,
{
    op: O,
    tcp_write: RefCell<OwnedWriteHalf>,
}

impl<O: OpDelta, T: Clone + Serialize> TcpComp<O, T>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<T>>,
    <O::LatRepr as LatticeRepr>::Repr: Collection<T, ()>,
{
    pub fn new(op: O, tcp_write: OwnedWriteHalf) -> Self {
        Self {
            op,
            tcp_write: RefCell::new(tcp_write),
        }
    }
}

impl<O: OpDelta, T: Clone + Serialize> Comp for TcpComp<O, T>
where
    O::LatRepr: LatticeRepr<Lattice = SetUnion<T>>,
    <O::LatRepr as LatticeRepr>::Repr: Collection<T, ()>,
{
    type Error = tokio::io::Error;

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            if let Some(hide) = (Next { op: &self.op }).await {
                for item in hide.reveal_ref().keys() {
                    let bytes = serde_json::to_vec(item)?;
                    let mut tcp_write_mut = self.tcp_write.borrow_mut();
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
