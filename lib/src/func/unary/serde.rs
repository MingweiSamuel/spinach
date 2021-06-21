use std::any::{Any, TypeId};

use bytes::{BytesMut, BufMut};
use serde::de::DeserializeOwned;

use crate::hide::{Hide, Qualifier};
use crate::lattice::{LatticeRepr};
use crate::lattice::bytes::BytesRepr;
use crate::lattice::bottom::BottomRepr;

use super::Morphism;

pub struct Deserialize<Lr: Any + LatticeRepr>
where
    Lr::Repr: DeserializeOwned,
{
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: Any + LatticeRepr> Deserialize<Lr>
where
    Lr::Repr: DeserializeOwned,
{
    const TYPE_ID: u64 = std::intrinsics::type_id::<Lr>();

    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Lr: Any + LatticeRepr> Default for Deserialize<Lr>
where
    Lr::Repr: DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Lr: Any + LatticeRepr> Morphism for Deserialize<Lr>
where
    Lr::Repr: DeserializeOwned,
{
    type InLatRepr  = BytesRepr<Lr>;
    type OutLatRepr = BottomRepr<Lr>;

    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        let bytes = item.into_reveal();

        let out = match bincode::deserialize::<'_, (u64, Lr::Repr)>(&*bytes) {
            Ok((tid, repr)) => {
                if Self::TYPE_ID == tid {
                    Some(repr)
                }
                else {
                    eprintln!("Invalid TypeId, expected: {}, found: {}.", Self::TYPE_ID, tid);
                    None
                }
            }
            Err(err) => {
                eprintln!("Failed to parse msg: {:?}, error: {}", bytes, err);
                None
            }
        };
        Hide::new(out)
    }
}



pub struct Serialize<Lr: Any + LatticeRepr>
where
    Lr::Repr: serde::ser::Serialize,
{
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: Any + LatticeRepr> Serialize<Lr>
where
    Lr::Repr: serde::ser::Serialize,
{
    const TYPE_ID: u64 = std::intrinsics::type_id::<Lr>();

    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Lr: Any + LatticeRepr> Morphism for Serialize<Lr>
where
    Lr::Repr: serde::ser::Serialize,
{
    type InLatRepr  = Lr;
    type OutLatRepr = BytesRepr<Lr>;

    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        let item = (Self::TYPE_ID, item.into_reveal());

        let mut writer = BytesMut::new().writer();
        bincode::serialize_into(&mut writer, &item).expect("Failed to serialize");
        let bytes = writer.into_inner().freeze();

        Hide::new(bytes)
    }
}
