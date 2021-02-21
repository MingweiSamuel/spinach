pub trait PureFn {
    type Indomain<'a>;
    type Outdomain<'a>;
    fn call<'a>(&self, item: Self::Indomain<'a>) -> Self::Outdomain<'a>;
}

// pub trait PureRefFn {
//     type Indomain;
//     type Outdomain;
//     fn call(&self, item: &Self::Indomain) -> Self::Outdomain;
// }
