use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use spinach::tokio::stream::{ StreamExt };
use spinach::tokio::sync::mpsc;
use spinach::tokio::task;

use spinach::{ Lattice, MergeIntoLattice };
use spinach::merge::{ MaxMerge, MapUnionMerge };

// To run:
// cargo test -- --nocapture

#[tokio::test]
async fn my_test() {
    // Make a uszie -> String map lattice.
    let kvs: Lattice<HashMap<usize, Lattice<String, MaxMerge>>, MapUnionMerge> = Default::default();
    // Put it in a refcell so I don't have to worry about ownership.
    let kvs: RefCell<_> = RefCell::new(kvs);
    // Use RC to handle lifetimes.
    let kvs: Rc<_> = Rc::new(kvs);

    // Make a second RC'd pointer to send to merge_into.
    let kvs_clone = kvs.clone();

    let bufsize = 4;
    let (sender, reciever) = mpsc::channel::<usize>(bufsize);

    // Create a local task set so all the futures get run on the current,
    // so I don't have to worry about multiple threads.
    let local = task::LocalSet::new();


    // Pretend this is a task for the actor. It reads from the receiver, runs
    // those values through the pipeline, and sends them to the KVS sink.
    // (Using this strat, we'd probably need a task per sink per actor).
    local.spawn_local(
        reciever.map(|x| x * x)
            .filter(|x| *x > 10)
            .map(|x| (x, format!("Hello I am {} :)", x).into()))
            .map(|(k, v)| {
                let mut y: HashMap<_, _> = Default::default();
                y.insert(k, v);
                y
            })
            .merge_into(kvs_clone));

    {
        let sender = sender;
        // Now pretend these are messages from the outside world.
        let mut outside_sender = sender.clone();
        local.spawn_local(async move {
            outside_sender.send( 6).await.unwrap();
            outside_sender.send( 1).await.unwrap();
            outside_sender.send( 2).await.unwrap();
            outside_sender.send(10).await.unwrap();
        });
        // And just another task so we can mix messages up.
        let mut outside_sender = sender.clone();
        local.spawn_local(async move {
            outside_sender.send( 5).await.unwrap();
            outside_sender.send(10).await.unwrap();
            outside_sender.send( 9).await.unwrap();
        });
    }

    // At this point everything is already running.
    // In reality we would stop here and let things run forever, I guess.
    // But let's stop the system and see what happened.

    // Wait for the actor to finish processing everything.
    local.await;

    // Print out the resulting map.
    println!("{:#?}", kvs.borrow_mut().reveal());
}
