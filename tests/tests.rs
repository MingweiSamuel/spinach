use std::collections::HashMap;

use spinach::op::*;
use spinach::merge::{ Merge, MapUnion, DominatingPair, Max };



pub struct IncrementFn;
impl PureFn for IncrementFn {
    type Indomain = usize;
    type Outdomain = Option<usize>;
    fn call(&self, item: Self::Indomain) -> Self::Outdomain {
        Some(item + 1)
    }
}



#[tokio::test]
pub async fn test_cycle_channel() -> Result<(), String> {

    let ( push_pipe, pull_pipe ) = channel_op::<usize>(10);
    let mut push_pipe = push_pipe;

    let pull_pipe = MapFilterMoveOp::new(pull_pipe, IncrementFn);
    let pull_pipe = DebugOp::new("channel", pull_pipe);

    push_pipe.push(350).await;
    push_pipe.push(650).await;

    let mut comp = StaticComp::new(pull_pipe, push_pipe);
    for _ in 0_usize..10 {
        comp.tick_moveop().await;
    }

    Ok(())
}

#[tokio::test]
pub async fn test_cycle_handoff() -> Result<(), String> {

    let ( push_pipe, pull_pipe ) = handoff_op::<DF<usize>>();
    let mut push_pipe = push_pipe;

    let pull_pipe = MapFilterMoveOp::new(pull_pipe, IncrementFn);
    let pull_pipe = DebugOp::new("handoff", pull_pipe);

    push_pipe.push(150).await;

    let mut comp = StaticComp::new(pull_pipe, push_pipe);
    for _ in 0_usize..10 {
        comp.tick_moveop().await;
    }

    Ok(())
}

#[tokio::test]
pub async fn test_kvs() -> Result<(), String> {

    type MyLattice = MapUnion<HashMap<
        &'static str,
        DominatingPair<Max<usize>, Max<&'static str>>
    >>;

    let ( mut write_pipe, pull_pipe ) = channel_op::<(&'static str, usize, &'static str)>(10);


    struct TupleToHashMapFn;
    impl PureFn for TupleToHashMapFn {
        type Indomain = ( &'static str, usize, &'static str );
        type Outdomain = Option<<MyLattice as Merge>::Domain>;
        fn call(&self, ( k, t, v ): Self::Indomain) -> Self::Outdomain {
            let mut map = HashMap::new();
            map.insert(k, ( t, v ));
            Some(map)
        }
    }

    let pull_pipe = MapFilterMoveOp::new(pull_pipe, TupleToHashMapFn);
    let pull_pipe = LatticeOp::<_, MyLattice>::new_default(pull_pipe);


    struct ReadKeyFn {
        key: &'static str,
    }
    impl PureRefFn for ReadKeyFn {
        type Indomain = <MyLattice as Merge>::Domain;
        type Outdomain = Option<&'static str>;
        fn call(&self, map: &Self::Indomain) -> Self::Outdomain {
            map.get(self.key).map(|opt| opt.1)
        }
    }


    let read_foo_0 = NullOp::<RX<Max<&'static str>>>::new();
    let read_foo_0 = DebugOp::new("foo 0", read_foo_0);
    let read_foo_0 = MapFoldRefOp::new(read_foo_0, ReadKeyFn { key: "foo" });

    let read_foo_1 = NullOp::<RX<Max<&'static str>>>::new();
    let read_foo_1 = DebugOp::new("foo 1", read_foo_1);
    let read_foo_1 = MapFoldRefOp::new(read_foo_1, ReadKeyFn { key: "foo" });


    unimplemented!("FIX ME FOR MONOTONIC SAFETY!");


    // let comp = DynComp::<MyLattice, _, _>::new(pull_pipe);
    // // let comp = StaticComp::new(pull_pipe, read_foo_pipe);


    // write_pipe.push(("foo", 200, "bar")).await;
    // write_pipe.push(("foo", 100, "baz")).await;


    // let mut comp = comp;
    // let read_foo_0 = read_foo_0;
    // for _ in 0_usize..10 {
    //     comp.tick_refop().await;
    // }
    // comp.add_split(read_foo_0).await;
    // for _ in 0_usize..10 {
    //     comp.tick_refop().await;
    // }

    // write_pipe.push(("foo", 300, "ding")).await;

    // comp.add_split(read_foo_1).await;



    Ok(())
}
