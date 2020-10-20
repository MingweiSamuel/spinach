use super::{ Pipe, Unconnected };


pub struct ChainedUnconnected<P, Q>
where
    P: Unconnected<NextPipe = <Q as Unconnected>::ThisPipe>,
    Q: Unconnected,
{
    first: P,
    second: Q,
}
impl <P, Q> ChainedUnconnected<P, Q>
where
    P: Unconnected<NextPipe = <Q as Unconnected>::ThisPipe>,
    Q: Unconnected,
{
    pub fn new(first: P, second: Q) -> Self {
        Self {
            first: first,
            second: second,
        }
    }
}
impl <P, Q> Unconnected for ChainedUnconnected<P, Q>
where
    P: Unconnected<NextPipe = <Q as Unconnected>::ThisPipe>,
    Q: Unconnected,
{
    type ThisPipe = <P as Unconnected>::ThisPipe;
    type NextPipe = <Q as Unconnected>::NextPipe;

    fn connect(self, next_pipe: Self::NextPipe) -> Self::ThisPipe {
        let next_pipe = self.second.connect(next_pipe);
        self.first.connect(next_pipe)
    }
}


use super::{ UnconnectedPipe, MapPipe };

pub struct Builder<U> {
    unconnected: U,
}
impl Builder<()> {
    pub fn new() -> Self {
        Self {
            unconnected: (),
        }
    }
    pub fn map<T, F, P>(self, mapper: F) -> Builder<UnconnectedPipe<MapPipe<T, F, P>>>
    where
        F: Fn(T) -> <P as Pipe>::Item,
        P: Pipe,
    {
        Builder {
            unconnected: UnconnectedPipe::<MapPipe<T, F, P>>::new(mapper),
        }
    }
}

impl <U, T, F, P> Builder<U>
where
    U: Unconnected<NextPipe = MapPipe<T, F, P>>,
    F: Fn(T) -> <P as Pipe>::Item,
    P: Pipe,
{
    pub fn map(self, mapper: F) -> Builder<ChainedUnconnected<U, UnconnectedPipe<MapPipe<T, F, P>>>> {
        Builder {
            unconnected: ChainedUnconnected::new(self.unconnected, UnconnectedPipe::<MapPipe<T, F, P>>::new(mapper)),
        }
    }
}

impl <U> Builder<U>
where
    U: Unconnected
{
    pub fn connect(self, tank: <U as Unconnected>::NextPipe) -> <U as Unconnected>::ThisPipe
    {
        self.unconnected.connect(tank)
    }
}

pub fn test2() {
    use std::collections::HashSet;

    use crate::Lattice;
    use crate::merge::{ UnionMerge };
    use super::{ Tank };

    let tank = Tank::new(Lattice::<HashSet<String>, UnionMerge>::default());

    let builder = Builder::new();
    let builder = builder
        .map(|x: usize| x + 10)
        .map(|x| format!("Hello {} :)", x - 2))
        .map(|x| {
            let mut y = HashSet::new();
            y.insert(x);
            y
        })
        .connect(tank);
    let _ = builder;
}


///
pub fn test() {
    use std::collections::HashMap;

    use crate::Lattice;
    use crate::merge::{ MaxMerge, MinMerge, DominatingPairMerge, MapUnionMerge };

    use super::{ Tank, FlattenPipe, FilterPipe };

    type VersionedString = Lattice<
        (Lattice<usize, MaxMerge>, Lattice<&'static str, MinMerge>),
        DominatingPairMerge>;
    type AnnaMap = Lattice<
        HashMap<&'static str, VersionedString>,
        MapUnionMerge>;

    let tank = Tank::new(AnnaMap::default());

    type Triple = ( &'static str, usize, &'static str );
    let builder = UnconnectedPipe::<FilterPipe<_, _>>::new(|( _, _, v ): &Triple| !v.contains("Christ"));
    let builder = ChainedUnconnected::new(builder, UnconnectedPipe::<MapPipe<_, _, _>>::new(|( k, t, v )| {
        vec![ ( k, t, v ), ( k, t - 1, "other str" ) ]
    }));
    let builder = ChainedUnconnected::new(builder, UnconnectedPipe::<FlattenPipe<_, _>>::new(()));
    let builder = ChainedUnconnected::new(builder, UnconnectedPipe::<MapPipe<_, _, _>>::new(|( k, t, v ): Triple| {
        let mut y: HashMap<_, _> = Default::default();
        y.insert(k, ( t.into(), v.into() ).into());
        y
    }));

    let _final_pipe = builder.connect(tank);
}