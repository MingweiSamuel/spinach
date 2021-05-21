use std::cell::RefCell;
use std::task::{Context, Poll};
use std::pin::Pin;

use futures_core::stream::Stream;
use serde::de::DeserializeOwned;
use tokio::net::tcp::OwnedReadHalf;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

use crate::collections::{Single};
use crate::hide::{Hide, Delta};
use crate::lattice::setunion::{SetUnionRepr};
use crate::metadata::Order;
use crate::tag;

use super::*;

pub struct TcpOp<T: Clone + DeserializeOwned> {
    _phantom: std::marker::PhantomData<T>,
    framed_read: RefCell<Pin<Box<FramedRead<OwnedReadHalf, LengthDelimitedCodec>>>>,
}

impl<T: Clone + DeserializeOwned> TcpOp<T> {
    pub fn new(tcp_read: OwnedReadHalf) -> Self {
        let framed_read = LengthDelimitedCodec::builder()
            .length_field_length(2)
            .new_read(tcp_read);
        Self {
            _phantom: std::marker::PhantomData,
            framed_read: RefCell::new(Box::pin(framed_read)),
        }
    }
}

impl<T: Clone + DeserializeOwned> Op for TcpOp<T> {
    type LatRepr = SetUnionRepr<tag::SINGLE, T>;
}

impl<T: Clone + DeserializeOwned> OpDelta for TcpOp<T> {
    type Ord = TcpOrderInvalid;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.framed_read
            .borrow_mut()
            .as_mut()
            .poll_next(ctx)
        {
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => {
                println!("!!!0 {:?}", err);
                Poll::Pending
            }
            Poll::Ready(Some(Ok(bytes))) => {
                match serde_json::from_slice(&*bytes) {
                    Ok(val) => {
                        let val = Hide::new(Single(val));
                        Poll::Ready(Some(val))
                    }
                    Err(err) => {
                        println!("!!!1 {:?}", err);
                        Poll::Pending
                    }
                }
            }
            Poll::Pending => Poll::Pending
        }
    }
}

pub enum TcpOrderInvalid {}
impl Order for TcpOrderInvalid {}
