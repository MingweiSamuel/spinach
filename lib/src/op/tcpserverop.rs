use std::net::SocketAddr;
use std::task::{Context, Poll};

use bytes::Bytes;

use crate::collections::{Single};
use crate::hide::{Hide, Delta};
use crate::lattice::LatticeRepr;
use crate::lattice::map_union::{MapUnionRepr};
use crate::metadata::Order;
use crate::tag;
use crate::tcp_server::TcpServer;

use super::optrait::*;

pub struct TcpServerOp<Lr: LatticeRepr<Repr = Bytes>> {
    tcp_server: TcpServer,
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr<Repr = Bytes>> TcpServerOp<Lr> {
    pub fn new(tcp_server: TcpServer) -> Self {
        Self {
            tcp_server,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Lr: LatticeRepr<Repr = Bytes>> Op for TcpServerOp<Lr> {
    type LatRepr = MapUnionRepr<tag::SINGLE, SocketAddr, Lr>;
}

pub enum TcpOrder {}
impl Order for TcpOrder {}

impl<Lr: LatticeRepr<Repr = Bytes>> OpDelta for TcpServerOp<Lr> {
    type Ord = TcpOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {

        match self.tcp_server.poll_accept(ctx) {
            Poll::Ready(Ok(addr)) => println!("New client! {}", addr),
            Poll::Ready(Err(err)) => eprintln!("Accept err! {}", err),
            Poll::Pending => (),
        }

        match self.tcp_server.poll_read(ctx) {
            Poll::Ready(Some((addr, bytes_mut))) => {
                let hide = Hide::new(Single((addr, bytes_mut.freeze())));
                Poll::Ready(Some(hide))
            }
            _ => Poll::Pending,
        }
    }
}
