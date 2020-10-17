use std::collections::{ HashMap, hash_map::DefaultHasher };
use std::hash::{ Hash, Hasher };

use spinach::tokio::stream::{ StreamExt };
use spinach::tokio::sync::mpsc;
use spinach::tokio::task;

use spinach::{ Lattice };
use spinach::merge::{ MaxMerge, MinMerge, MapUnionMerge, DominatingPairMerge };

use spinach::pipe::{ Tank, Pipe };
use spinach::pipe::{ SplitPipe, MapPipe, FilterPipe, }; //MergePipe, FlattenPipe };

// To run:
// `cargo test pipe_test -- --nocapture`

#[tokio::test]
async fn pipe_test() {
    type VersionedString = Lattice<
        (Lattice<usize, MaxMerge>, Lattice<&'static str, MinMerge>),
        DominatingPairMerge>;
    type AnnaMap = Lattice<
        HashMap<&'static str, VersionedString>,
        MapUnionMerge>;


    let tank_one = Tank::new(AnnaMap::default());
    let tank_two = Tank::new(AnnaMap::default());


    let local = task::LocalSet::new();


    let (sender, mut receiver) =
        mpsc::channel::<(&'static str, usize, &'static str)>(4);

    let tank_one_kv = tank_one.kv_pipe();
    let tank_two_kv = tank_two.kv_pipe();
    local.spawn_local(async move {
        let pipe = SplitPipe::new(tank_one_kv, tank_two_kv, |( key, _ )| {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            0 == hasher.finish() % 2
        });
        let pipe = MapPipe::new(pipe, |( k, t, v ): ( &'static str, usize, &'static str )| {
            ( k, ( t.into(), v.into() ).into() )
        });
        let pipe = FilterPipe::new(pipe, |( _, _, v )| !v.contains("Christ"));

        // Push things through the pipe (instead of pulling).
        while let Some(item) = receiver.next().await {
            pipe.merge_in(item);
        };
    });

    {
        // Disclaimer: I picked keys that would evenly distribute.
        let sender = sender;

        let outside_sender = sender.clone();
        local.spawn_local(async move {
            outside_sender.send(("chancellor", 2017, "Carol T. Christ"))
                .await.unwrap();
            outside_sender.send(("chancellor", 2004, "Robert J. Birgeneau"))
                .await.unwrap();
            outside_sender.send(("trillion_usd_company", 2018, "AAPL"))
                .await.unwrap();
            outside_sender.send(("dog", 1622, "German Shepard"))
                .await.unwrap();

            outside_sender.send(("ml_framework", 0, "Tensorflow"))
                .await.unwrap();
            outside_sender.send(("ml_framework", 0, "PyTorch"))
                .await.unwrap();
        });

        let outside_sender = sender.clone();
        local.spawn_local(async move {
            outside_sender.send(("chancellor", 2013, "Nicholas B. Dirks"))
                .await.unwrap();
            outside_sender.send(("trillion_usd_company", 2018, "AMZN"))
                .await.unwrap();

            outside_sender.send(("dog", 1706, "Dachshund"))
                .await.unwrap();
            outside_sender.send(("dog", 1200, "Greyhound"))
                .await.unwrap();
        });

        let outside_sender = sender.clone();
        local.spawn_local(async move {
            outside_sender.send(("dog", 1200, "Greyhound"))
                .await.unwrap();

            outside_sender.send(("ml_framework", 0, "Keras"))
                .await.unwrap();
        });
    }

    local.await;

    println!("{:#?}", tank_one.get_lattice().borrow().reveal());
    println!("{:#?}", tank_two.get_lattice().borrow().reveal());
}
