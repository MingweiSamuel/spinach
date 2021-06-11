use std::cell::RefCell;
use std::task::{Context, Poll};
use std::pin::Pin;

use bytes::BytesMut;
use futures_core::stream::Stream;
use tokio::net::tcp::OwnedReadHalf;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

use crate::collections::{Single};
use crate::hide::{Hide, Delta};
use crate::lattice::setunion::{SetUnionRepr};
use crate::metadata::Order;
use crate::tag;

use super::*;

pub struct TcpOp {
    framed_read: RefCell<FramedRead<OwnedReadHalf, LengthDelimitedCodec>>,
}

impl TcpOp {
    pub fn new(tcp_read: OwnedReadHalf) -> Self {
        let framed_read = LengthDelimitedCodec::builder()
            .length_field_length(2)
            .new_read(tcp_read);
        Self {
            framed_read: RefCell::new(framed_read),
        }
    }
}

impl Op for TcpOp {
    type LatRepr = SetUnionRepr<tag::SINGLE, BytesMut>;
}

pub enum TcpOrder {}
impl Order for TcpOrder {}

impl OpDelta for TcpOp {
    type Ord = TcpOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match Pin::new(&mut *self.framed_read.borrow_mut()).poll_next(ctx) {
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => {
                println!("!!!0 {:?}", err);
                Poll::Pending
            }
            Poll::Ready(Some(Ok(bytes))) => {
                let item = Hide::new(Single(bytes));
                Poll::Ready(Some(item))
            }
            Poll::Pending => Poll::Pending
        }
    }
}
