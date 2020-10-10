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
    let kvs: Lattice<HashMap<usize, Lattice<String, MaxMerge>>, MapUnionMerge> = Lattice::default();
    // Put it in a refcell so I don't have to worry about ownership.
    let kvs: RefCell<_> = RefCell::new(kvs);
    // Use RC to handle lifetimes.
    let kvs: Rc<_> = Rc::new(kvs);

    // Make a second RC'd pointer to send to merge_into.
    let kvs_clone = kvs.clone();


    // Create a local task set so I don't have to worry about threads.
    let local = task::LocalSet::new();
    local.run_until(async move {

        let bufsize = 4; // Doesn't really matter since we're on one thread...
        // MPSC is really not needed since we're on a single thread...
        let (mut sender, reciever) = mpsc::channel::<usize>(bufsize);

        // THE ACTUAL code ...
        reciever.map(|x| x * x)
            .filter(|x| *x > 10)
            .map(|x| (x, format!("Hello I am {} :)", x).into()))
            .map(|(k, v)| {
                let mut y: HashMap<_, _> = Default::default();
                y.insert(k, v);
                y
            })
            .merge_into(kvs_clone); // !! This actually opaquely spawns a task to poll...

        // Now pretend this is coming from the outside world somewhere
        sender.send( 6).await.unwrap();
        sender.send( 1).await.unwrap(); // Bad style to call .unwrap().
        sender.send( 2).await.unwrap();
        sender.send(10).await.unwrap();
        sender.send( 5).await.unwrap();
        sender.send(10).await.unwrap();
        sender.send( 9).await.unwrap();
    }).await;

    // !! Wait for the opaque task to finish merging everything...
    local.await;

    // Print out the resulting map.
    println!("{:#?}", kvs.borrow_mut().reveal());
}
