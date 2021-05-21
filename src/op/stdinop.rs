use std::cell::RefCell;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{Stdin, BufReader, Lines, AsyncBufReadExt};

use crate::collections::Single;
use crate::hide::{Hide, Delta};
use crate::tag::{SINGLE};
use crate::lattice::setunion::{SetUnionRepr};
use crate::metadata::Order;

use super::*;

pub struct StdinOp {
    reader: RefCell<Lines<BufReader<Stdin>>>,
}

impl StdinOp {
    pub fn new() -> Self {
        Self {
            reader: RefCell::new(BufReader::new(tokio::io::stdin()).lines()),
        }
    }
}

impl Op for StdinOp {
    type LatRepr = SetUnionRepr<SINGLE, String>;
}

impl OpDelta for StdinOp {
    type Ord = UserInputOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        loop {
            match Pin::new(&mut *self.reader.borrow_mut()).as_mut().poll_next_line(ctx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Result::Ok(opt)) => return Poll::Ready(opt.map(|x| Hide::new(Single(x)))),
                Poll::Ready(Result::Err(err)) => println!("ERROR: {}", err),
            }
        }
    }
}

pub struct UserInputOrder;
impl Order for UserInputOrder {}