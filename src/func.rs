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
