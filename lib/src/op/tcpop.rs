use std::cell::RefCell;
use std::task::{Context, Poll};
use std::pin::Pin;

use bytes::{Bytes};
use futures_core::stream::Stream;
use tokio::net::tcp::OwnedReadHalf;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

use crate::hide::{Hide, Delta};
use crate::lattice::LatticeRepr;
use crate::metadata::Order;

use super::optrait::*;

pub struct TcpOp<Lr: LatticeRepr<Repr = Bytes>> {
    framed_read: RefCell<FramedRead<OwnedReadHalf, LengthDelimitedCodec>>,
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr<Repr = Bytes>> TcpOp<Lr> {
    pub fn new(tcp_read: OwnedReadHalf) -> Self {
        let framed_read = LengthDelimitedCodec::builder()
            .length_field_length(2)
            .new_read(tcp_read);
        Self {
            framed_read: RefCell::new(framed_read),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Lr: LatticeRepr<Repr = Bytes>> Op for TcpOp<Lr> {
    type LatRepr = Lr;
}

pub enum TcpOrder {}
impl Order for TcpOrder {}

impl<Lr: LatticeRepr<Repr = Bytes>> OpDelta for TcpOp<Lr> {
    type Ord = TcpOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match Pin::new(&mut *self.framed_read.borrow_mut()).poll_next(ctx) {
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => {
                println!("!!!0 {:?}", err);
                Poll::Pending
            }
            Poll::Ready(Some(Ok(bytes_mut))) => {
                let item = Hide::new(bytes_mut.freeze());
                Poll::Ready(Some(item))
            }
            Poll::Pending => Poll::Pending
        }
    }
}
