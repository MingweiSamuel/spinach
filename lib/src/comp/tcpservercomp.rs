use std::any::Any;
use std::future::Future;
use std::net::SocketAddr;

use bincode::{Error, ErrorKind};
use serde::ser::Serialize;

use crate::lattice::{LatticeRepr};
use crate::lattice::map_union::{MapTag, MapUnionRepr};
use crate::op::OpDelta;
use crate::tcp_server::TcpServer;
use crate::tcp_server::serde::serialize;

use super::{Comp, Next};

pub struct TcpServerComp<O: OpDelta, Tag, Lr: Any + LatticeRepr>
where
    Tag: MapTag<SocketAddr, Lr::Repr>,
    MapUnionRepr<Tag, SocketAddr, Lr>: LatticeRepr,
    O: OpDelta<LatRepr = MapUnionRepr<Tag, SocketAddr, Lr>>,
    <O::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = (SocketAddr, Lr::Repr)>,
    Lr::Repr: Serialize,
{
    op: O,
    tcp_server: TcpServer,
    _phantom: std::marker::PhantomData<(Tag, Lr)>,
}

impl<O: OpDelta, Tag, Lr: Any + LatticeRepr> TcpServerComp<O, Tag, Lr>
where
    Tag: MapTag<SocketAddr, Lr::Repr>,
    MapUnionRepr<Tag, SocketAddr, Lr>: LatticeRepr,
    O: OpDelta<LatRepr = MapUnionRepr<Tag, SocketAddr, Lr>>,
    <O::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = (SocketAddr, Lr::Repr)>,
    Lr::Repr: Serialize,
{
    pub fn new(op: O, tcp_server: TcpServer) -> Self {
        Self {
            op,
            tcp_server,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<O: OpDelta, Tag, Lr: Any + LatticeRepr> Comp for TcpServerComp<O, Tag, Lr>
where
    Tag: MapTag<SocketAddr, Lr::Repr>,
    MapUnionRepr<Tag, SocketAddr, Lr>: LatticeRepr,
    O: OpDelta<LatRepr = MapUnionRepr<Tag, SocketAddr, Lr>>,
    <O::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = (SocketAddr, Lr::Repr)>,
    Lr::Repr: Serialize,
{
    type Error = Error;

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            if let Some(hide) = (Next { op: &self.op }).await {
                for (addr, repr) in hide.into_reveal().into_iter() {
                    let bytes = serialize::<Lr>(repr)?.freeze();
                    self.tcp_server.write(addr, bytes).await?;
                }
                Ok(())
            }
            else {
                Err(Box::new(ErrorKind::Custom("End of stream.".to_owned())))
            }
        }
    }
}
