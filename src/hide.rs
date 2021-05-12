use crate::lattice::LatticeRepr;

use ref_cast::RefCast;

pub trait Type {}
pub enum Delta {}
impl Type for Delta {}
pub enum Value {}
impl Type for Value {}

#[derive(RefCast)]
#[repr(transparent)]
pub struct Hide<Y: Type, Lr: LatticeRepr + ?Sized> {
    value: Lr::Repr,
    _phantom: std::marker::PhantomData<Y>,
}

impl<Y: Type, Lr: LatticeRepr + ?Sized> Hide<Y, Lr> {
    pub fn new(value: Lr::Repr) -> Self {
        Self {
            value,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn into_reveal(self) -> Lr::Repr {
        self.value
    }

    pub fn as_reveal(&self) -> &Lr::Repr {
        &self.value
    }

    pub fn as_reveal_mut(&mut self) -> &mut Lr::Repr {
        &mut self.value
    }

    pub fn into_reveal_value(self) -> Hide<Value, Lr> {
        Hide {
            value: self.value,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn into_delta(self) -> Hide<Delta, Lr> {
        Hide {
            value: self.value,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Y: Type, Lr: LatticeRepr> Clone for Hide<Y, Lr> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}
