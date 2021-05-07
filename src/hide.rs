use crate::lattice::LatticeRepr;

use ref_cast::RefCast;

pub trait Type {}
pub enum Delta {}
impl Type for Delta {}
pub enum Value {}
impl Type for Value {}

#[derive(RefCast)]
#[repr(transparent)]
pub struct Hide<Y: Type, LR: LatticeRepr> {
    value: LR::Repr,
    _phantom: std::marker::PhantomData<Y>,
}

impl<Y: Type, LR: LatticeRepr> Hide<Y, LR> {
    pub fn new(value: LR::Repr) -> Self {
        Self {
            value,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn into_reveal(self) -> LR::Repr {
        self.value
    }

    pub fn as_reveal(&self) -> &LR::Repr {
        &self.value
    }
}

impl<LR: LatticeRepr> std::ops::Deref for Hide<Value, LR> {
    type Target = Hide<Delta, LR>;

    fn deref(&self) -> &Self::Target {
        Hide::ref_cast(&self.value)
    }
}
