use std::collections::HashMap;

use spinach::op2::*;
use spinach::merge::{ Merge, MapUnion, DominatingPair, Max };

#[tokio::test]
pub async fn test_cycle_channel() -> Result<(), String> {

    let ( push_pipe, pull_pipe ) = channel_op::<usize>(10);
    let mut push_pipe = push_pipe;

    let pull_pipe = MapFilterMoveOp::<_, _, usize>::new(pull_pipe, |x: usize| Some(x + 1));
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

    let ( push_pipe, pull_pipe ) = handoff_op::<DF, usize>();
    let mut push_pipe = push_pipe;

    let pull_pipe = MapFilterMoveOp::<_, _, usize>::new(pull_pipe, |x: usize| Some(x + 1));
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

    let pull_pipe = MapFilterMoveOp::<_, _, <MyLattice as Merge>::Domain>::new(pull_pipe, |( k, t, v ): ( &'static str, usize, &'static str )| {
        let mut map = HashMap::new();
        map.insert(k, ( t, v ));
        Some(map)
    });
    let pull_pipe = LatticeOp::<_, MyLattice>::new_default(pull_pipe);


    let read_foo_pipe = NullOp::<RX, &'static str>::new();
    let read_foo_pipe = DebugOp::new("foo 0", read_foo_pipe);
    let read_foo_pipe = MapFilterRefOp::<_, _, <MyLattice as Merge>::Domain>::new(read_foo_pipe,
        |map: &<MyLattice as Merge>::Domain| map.get("foo").map(|opt| opt.1));


    let mut comp = DynComp::<_, _>::new(pull_pipe);
    // let comp = StaticComp::new(pull_pipe, read_foo_pipe);


    write_pipe.push(("foo", 200, "bar")).await;
    write_pipe.push(("foo", 100, "baz")).await;



    comp.add_split(read_foo_pipe).await;

    
    // // write_pipe.push(("foo", 100, "baz"));

    for i in 0_usize..10 {
        comp.tick_refop().await;
        if i == 5 {    
            write_pipe.push(("foo", 200 + i, "ding")).await;
            sleep_yield_now().await;
        }
    }


    Ok(())
}
