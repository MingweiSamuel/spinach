use std::net::SocketAddr;
use std::task::{Context, Poll};

use bytes::BytesMut;

use crate::collections::{Single};
use crate::hide::{Hide, Delta};
use crate::lattice::setunion::{SetUnionRepr};
use crate::metadata::Order;
use crate::tag;
use crate::tcp_server::TcpServer;

use super::*;

pub struct TcpServerOp {
    tcp_server: TcpServer,
}

impl TcpServerOp {
    pub fn new(tcp_server: TcpServer) -> Self {
        Self { tcp_server }
    }
}

impl Op for TcpServerOp {
    type LatRepr = SetUnionRepr<tag::SINGLE, (SocketAddr, BytesMut)>;
}

pub enum TcpOrder {}
impl Order for TcpOrder {}

impl OpDelta for TcpServerOp {
    type Ord = TcpOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {

        match self.tcp_server.poll_accept(ctx) {
            Poll::Ready(Ok(addr)) => println!("New client! {}", addr),
            Poll::Ready(Err(err)) => eprintln!("Accept err! {}", err),
            Poll::Pending => (),
        }

        match self.tcp_server.poll_read(ctx) {
            Poll::Ready(Some(pair)) => {
                let hide = Hide::new(Single(pair));
                Poll::Ready(Some(hide))
            }
            _ => Poll::Pending,
        }
    }
}
