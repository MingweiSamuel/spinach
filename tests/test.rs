use std::collections::HashMap;

use spinach::tokio::stream::{ StreamExt };
use spinach::tokio::sync::mpsc;
use spinach::tokio::task;

use spinach::{ Lattice, MergeIntoLattice };
use spinach::merge::{ MaxMerge, MinMerge, MapUnionMerge, DominatingPairMerge };

// To run:
// cargo test -- --nocapture

#[tokio::test]
async fn actor_test() {
    // Alias for a versioned string with a simple counter clock.
    // Conflicting versions are resolved by taking the earlier string alphabetically (MinMerge).
    type VersionedString = Lattice<
        (Lattice<usize, MaxMerge>, Lattice<&'static str, MinMerge>),
        DominatingPairMerge>;
    // KVS for mapping strings to `VersionedString`s.
    let kvs: Lattice<HashMap<&'static str, VersionedString>, MapUnionMerge> = Default::default();

    // Channel for buffering messages.
    let bufsize = 4;
    let (sender, reciever) = mpsc::channel::<(&'static str, usize, &'static str)>(bufsize);

    // Create a local task set so all the futures get run on the current,
    // so I don't have to worry about multiple threads.
    let local = task::LocalSet::new();


    // Pretend this is a task for the actor. It reads from the receiver, runs
    // those values through the pipeline, and sends them to the KVS sink.
    // (Using this strat, we'd probably need a task per sink per actor).
    let kvs_result = local.spawn_local(
        reciever
            .filter(|(_, _, v)| !v.contains("Christ")) // No swearing allowed.
            .map(|(k, t, v)| {
                let mut y: HashMap<_, _> = Default::default();
                y.insert(k, (t.into(), v.into()).into());
                y
            })
            .merge_into(kvs));


    // Send some stuff.
    // Pretend these are messages from the outside world.
    {
        let sender = sender;
        let mut outside_sender = sender.clone();
        local.spawn_local(async move {
            outside_sender.send(("chancellor", 2017, "Carol T. Christ")).await.unwrap();
            outside_sender.send(("chancellor", 2004, "Robert J. Birgeneau")).await.unwrap();
            outside_sender.send(("trillion_usd_company", 2018, "AAPL")).await.unwrap();
        });
        // And just another task so we can mix messages up.
        let mut outside_sender = sender.clone();
        local.spawn_local(async move {
            outside_sender.send(("chancellor", 2013, "Nicholas B. Dirks")).await.unwrap();
            outside_sender.send(("trillion_usd_company", 2018, "AMZN")).await.unwrap();
        });
    }


    // At this point everything is already running.
    // In reality we would stop here and let things run forever, I guess.
    // But let's stop the system and see what happened.

    // Wait for the actor to finish processing everything.
    local.await;
    let kvs = kvs_result.await.expect("Failed to get final KVS.");

    // Print out the resulting map.
    println!("{:#?}", kvs.reveal());
    // Result:
    // {
    //     "chancellor": (
    //         2013,
    //         "Nicholas B. Dirks",
    //     ),
    //     "trillion_usd_company": (
    //         2018,
    //         "AAPL",
    //     ),
    // }
}
