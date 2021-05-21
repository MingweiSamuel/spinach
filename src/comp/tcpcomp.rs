use std::cell::RefCell;
use std::convert::TryInto;

use serde::ser::Serialize;
use tokio::io::{AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;

use crate::lattice::LatticeRepr;
use crate::op::OpDelta;

use super::Next;

pub struct TcpComp<O: OpDelta>
where
    <O::LatRepr as LatticeRepr>::Repr: Serialize,
{
    op: O,
    tcp_write: RefCell<OwnedWriteHalf>,
}

impl<O: OpDelta> TcpComp<O>
where
    <O::LatRepr as LatticeRepr>::Repr: Serialize,
{
    pub fn new(op: O, tcp_write: OwnedWriteHalf) -> Self {
        Self {
            op,
            tcp_write: RefCell::new(tcp_write),
        }
    }

    pub async fn tick(&self) -> tokio::io::Result<()> {
        if let Some(hide) = (Next { op: &self.op }).await {
            let bytes = serde_json::to_vec(hide.reveal_ref())?;
            let mut tcp_write_mut = self.tcp_write.borrow_mut();
            let len = bytes.len().try_into().unwrap_or_else(|_| panic!("Message too long! {}", bytes.len()));
            tcp_write_mut.write_u16(len).await?;
            tcp_write_mut.write_all(&*bytes).await?;
            Ok(())
        }
        else {
            Err(tokio::io::Error::new(std::io::ErrorKind::UnexpectedEof, "End of stream."))
        }
    }

    pub async fn run(&self) -> tokio::io::Result<!> {
        loop {
            self.tick().await?;
        }
    }
}
