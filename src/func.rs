pub trait PureFn {
    type Indomain;
    type Outdomain;
    fn call(&self, item: Self::Indomain) -> Self::Outdomain;
}

pub trait PureRefFn {
    type Indomain;
    type Outdomain;
    fn call(&self, item: &Self::Indomain) -> Self::Outdomain;
}
