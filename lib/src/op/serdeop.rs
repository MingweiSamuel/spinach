use std::any::Any;
use std::task::{Context, Poll};

use bytes::{BufMut, BytesMut};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use crate::hide::{Hide, Delta};
use crate::lattice::{LatticeRepr};
use crate::lattice::serial::SerialRepr;

use super::{Op, OpDelta};

pub struct SerializeOp<O: Op>
where
    O::LatRepr: Any,
    <O::LatRepr as LatticeRepr>::Repr: Serialize,
{
    op: O,
}

impl<O: Op> SerializeOp<O>
where
    O::LatRepr: Any,
    <O::LatRepr as LatticeRepr>::Repr: Serialize,
{
    const TYPE_ID: u64 = std::intrinsics::type_id::<O::LatRepr>();

    pub fn new(op: O) -> Self {
        Self { op }
    }
}


impl<O: Op> Op for SerializeOp<O>
where
    O::LatRepr: Any,
    <O::LatRepr as LatticeRepr>::Repr: Serialize,
{
    type LatRepr = SerialRepr<O::LatRepr>;
}


impl<O: OpDelta> OpDelta for SerializeOp<O>
where
    O::LatRepr: Any,
    <O::LatRepr as LatticeRepr>::Repr: Serialize,
{
    type Ord = O::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {
                let item = (Self::TYPE_ID, delta.into_reveal());

                let mut writer = BytesMut::new().writer();
                match bincode::serialize_into(&mut writer, &item) {
                    Ok(()) => {
                        let bytes = writer.into_inner().freeze();

                        Poll::Ready(Some(Hide::new(bytes)))
                    }
                    Err(e) => {
                        eprintln!("Failed to serialize, error: {}", e);
                        Poll::Pending
                    }
                }
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}


pub struct DeserializeOp<O: Op, Lr: Any + LatticeRepr>
where
    Lr::Repr: DeserializeOwned,
{
    op: O,
    _phantom: std::marker::PhantomData<Lr>,
}

impl<O, Lr: Any + LatticeRepr> DeserializeOp<O, Lr>
where
    O: Op<LatRepr = SerialRepr<Lr>>,
    Lr::Repr: DeserializeOwned,
{
    const TYPE_ID: u64 = std::intrinsics::type_id::<Lr>();

    pub fn new(op: O) -> Self {
        Self {
            op,
            _phantom: std::marker::PhantomData,
        }
    }
}


impl<O, Lr: Any + LatticeRepr> Op for DeserializeOp<O, Lr>
where
    O: Op<LatRepr = SerialRepr<Lr>>,
    Lr::Repr: DeserializeOwned,
{
    type LatRepr = Lr;
}

impl<O, Lr: Any + LatticeRepr> OpDelta for DeserializeOp<O, Lr>
where
    O: OpDelta<LatRepr = SerialRepr<Lr>>,
    Lr::Repr: DeserializeOwned,
{
    type Ord = O::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {

                let bytes = delta.into_reveal();
                match bincode::deserialize::<(u64, Lr::Repr)>(&*bytes) {
                    Ok((tid, repr)) => {
                        if Self::TYPE_ID == tid {
                            Poll::Ready(Some(Hide::new(repr)))
                        }
                        else {
                            eprintln!("Invalid TypeId, expected: {}, found: {}.", Self::TYPE_ID, tid);
                            Poll::Pending
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to parse msg: {:?}, error: {}", bytes, err);
                        Poll::Pending
                    }
                }
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
