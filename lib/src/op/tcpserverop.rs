use std::net::SocketAddr;
use std::task::{Context, Poll};

use serde::de::DeserializeOwned;

use crate::collections::{Single};
use crate::hide::{Hide, Delta};
use crate::lattice::setunion::{SetUnionRepr};
use crate::metadata::Order;
use crate::tag;
use crate::tcp_server::TcpServer;

use super::*;

pub struct TcpServerOp<T: Clone + DeserializeOwned> {
    tcp_pool: TcpServer,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + DeserializeOwned> TcpServerOp<T> {
    pub fn new(tcp_pool: TcpServer) -> Self {
        Self {
            tcp_pool,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Clone + DeserializeOwned> Op for TcpServerOp<T> {
    type LatRepr = SetUnionRepr<tag::SINGLE, (SocketAddr, T)>;
}

pub enum TcpOrder {}
impl Order for TcpOrder {}

impl<T: Clone + DeserializeOwned> OpDelta for TcpServerOp<T> {
    type Ord = TcpOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {

        match self.tcp_pool.poll_accept(ctx) {
            Poll::Ready(Ok(addr)) => println!("New client! {}", addr),
            Poll::Ready(Err(err)) => eprintln!("Accept err! {}", err),
            Poll::Pending => (),
        }

        match self.tcp_pool.poll_read(ctx) {
            Poll::Ready(Some(pair)) => {
                let hide = Hide::new(Single(pair));
                Poll::Ready(Some(hide))
            }
            _ => Poll::Pending,
        }
    }
}
