//! Function traits.

/// Represents a pure function, owned->owned values.
pub trait PureFn {
    type Indomain;
    type Outdomain;
    fn call(&self, item: Self::Indomain) -> Self::Outdomain;
}

/// Represents a pure function, reference->owned values.
pub trait PureRefFn {
    type Indomain;
    type Outdomain;
    fn call(&self, item: &Self::Indomain) -> Self::Outdomain;
}

pub trait PureRefRefFn {
    type Indomain;
    type Outdomain;
    fn call<'a>(&self, item: &'a Self::Indomain) -> &'a Self::Outdomain;
}

pub trait RendezvousFn {
    type InDf;
    type InRx;
    type Outdomain;
    fn call<'a>(&self, item: (Self::InDf, &'a Self::InRx)) -> Self::Outdomain;
}