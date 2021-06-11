use std::net::SocketAddr;
use std::task::{Context, Poll};

use serde::ser::Serialize;
use serde::de::DeserializeOwned;

use crate::collections::{Single};
use crate::hide::{Hide, Delta};
use crate::lattice::setunion::{SetUnionRepr};
use crate::metadata::Order;
use crate::tag;
use crate::tcp_pool::TcpPool;

use super::*;

pub struct TcpPoolOp<T: Clone + Serialize + DeserializeOwned> {
    tcp_pool: TcpPool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Serialize + DeserializeOwned> TcpPoolOp<T> {
    pub fn new(tcp_pool: TcpPool) -> Self {
        Self {
            tcp_pool,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Clone + Serialize + DeserializeOwned> Op for TcpPoolOp<T> {
    type LatRepr = SetUnionRepr<tag::SINGLE, (SocketAddr, T)>;
}

pub enum TcpOrder {}
impl Order for TcpOrder {}

impl<T: Clone + Serialize + DeserializeOwned> OpDelta for TcpPoolOp<T> {
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
