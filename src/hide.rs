use crate::lattice::LatticeRepr;

use ref_cast::RefCast;

pub trait Qualifier {}
pub enum Delta {}
impl Qualifier for Delta {}
pub enum Value {}
impl Qualifier for Value {}

#[derive(RefCast)]
#[repr(transparent)]
pub struct Hide<Y: Qualifier, Lr: LatticeRepr + ?Sized> {
    value: Lr::Repr,
    _phantom: std::marker::PhantomData<Y>,
}

impl<Y: Qualifier, Lr: LatticeRepr + ?Sized> Hide<Y, Lr> {
    pub fn new(value: Lr::Repr) -> Self {
        Self {
            value,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn into_reveal(self) -> Lr::Repr {
        self.value
    }

    pub fn reveal_ref(&self) -> &Lr::Repr {
        &self.value
    }

    pub fn reveal_mut(&mut self) -> &mut Lr::Repr {
        &mut self.value
    }

    pub fn into_delta(self) -> Hide<Delta, Lr> {
        Hide {
            value: self.value,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Y: Qualifier, Lr: LatticeRepr> Clone for Hide<Y, Lr> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}
