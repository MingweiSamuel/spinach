use std::cell::RefCell;
use std::task::{Context, Poll};

use nng::{Aio, AioResult, Message, Protocol, Socket};
use tokio::sync::mpsc;

use crate::collections::{Single};
use crate::hide::{Hide, Delta};
use crate::lattice::setunion::{SetUnionRepr};
use crate::metadata::Order;
use crate::tag;

use super::*;

pub struct NngOp {
    receiver: RefCell<mpsc::UnboundedReceiver<Message>>,
    _ctxs: Vec<nng::Context>,
    _aios: Vec<Aio>,
}

impl NngOp {
    pub fn new(url: &str, workers: usize) -> Self {
        // Create the tokio channel.
        let (sender, receiver) = mpsc::unbounded_channel();

        // Create the socket.
        let socket = Socket::new(Protocol::Rep0).unwrap();

        // Create all of the worker contexts.
        let ctxs: Vec<_> = (0..workers).map(|_| nng::Context::new(&socket).unwrap()).collect();
        let aios: Vec<_> = ctxs.iter()
            .map(|ctx| {
                let ctx_clone = ctx.clone();
                let sender_clone = sender.clone();
                Aio::new(move |aio, res| Self::worker_callback(aio, res, &ctx_clone, &sender_clone)).unwrap()
            })
            .collect();

        // Only after we have the workers do we start listening.
        socket.listen(url).unwrap();

        // Now start all of the workers listening.
        for (ctx, aio) in ctxs.iter().zip(aios.iter()) {
            ctx.recv(&aio).unwrap();
        }

        Self {
            receiver: RefCell::new(receiver),
            _ctxs: ctxs,
            _aios:aios,
        }
    }

    fn worker_callback(aio: Aio, res: AioResult, ctx: &nng::Context, sender: &mpsc::UnboundedSender<Message>) {
        match res {
            // We successfully sent the message, wait for a new one.
            AioResult::Send(Ok(())) => ctx.recv(&aio).unwrap(),

            // We successfully received a message.
            AioResult::Recv(Ok(msg)) => {
                sender.send(msg).unwrap();
                // Send an "ack" empty message
                ctx.send(&aio, Message::new()).unwrap();
            }

            // We successfully slept.
            AioResult::Sleep(Ok(())) => {
                panic!("Callback does not sleep.");
            }

            // Anything else is an error and we will just panic.
            AioResult::Send(Err((_, e))) | AioResult::Recv(Err(e)) | AioResult::Sleep(Err(e)) => {
                panic!("Error: {}", e)
            }
        }
    }
}

impl Op for NngOp {
    type LatRepr = SetUnionRepr<tag::SINGLE, Message>;
}

pub enum NngOrder {}
impl Order for NngOrder {}

impl OpDelta for NngOp {
    type Ord = NngOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.receiver.borrow_mut().poll_recv(ctx) {
            Poll::Ready(Some(msg)) => Poll::Ready(Some(Hide::new(Single(msg)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
