use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use tokio::sync::mpsc;

use spinach::pull::{ MoveNext, ChannelOp, LatticeOp2, MapFilterOp };
use spinach::merge::{ MapUnion, DominatingPair, Max };



pub async fn sleep_yield_now() {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{ Context, Poll };

    /// Yield implementation
    struct SleepYieldNow {
        yielded: bool,
    }

    impl Future for SleepYieldNow {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<()> {
            if self.yielded {
                return Poll::Ready(());
            }

            self.yielded = true;
            // cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    SleepYieldNow { yielded: false }.await
}



#[tokio::test]
pub async fn test_pull() -> Result<(), String> {

    type MyLattice = MapUnion<HashMap<
        &'static str,
        DominatingPair<Max<usize>, Max<&'static str>>
    >>;

    let ( sender, receiver ) = mpsc::channel::<( &'static str, usize, &'static str )>(1);

    let pipe = ChannelOp::new(receiver);
    let pipe = MapFilterOp::new(pipe, |( k, t, v )| {
        let mut hashmap = HashMap::new();
        hashmap.insert(k, ( t, v ));
        Some(hashmap)
    });
    let pipe = LatticeOp2::<_, MyLattice>::new(pipe, HashMap::new());

    let pipe_read_foo = MapFilterOp::new(pipe.clone(),
        |rc: Rc<RefCell<HashMap<_, _>>>| Some(rc.borrow().get("foo").cloned()));

    let worker = tokio::task::LocalSet::new();
    worker.spawn_local(async move {
        let mut pipe_read_foo = pipe_read_foo;

        while let Some(val) = MoveNext::new(&mut pipe_read_foo).await {
            println!("foo: {:?}", val);
            sleep_yield_now().await;
            // tokio::task::yield_now().await;
        }
    });


    let sender_clone = sender.clone();
    worker.spawn_local(async move {
        let sender = sender_clone;
        sender.send(( "xyz", 10, "vvv" )).await.unwrap();
        sender.send(( "foo", 10, "bar" )).await.unwrap();
        sender.send(( "bin", 11, "bag" )).await.unwrap();
        sender.send(( "foo", 12, "baz" )).await.unwrap();
        sender.send(( "xyz", 13, "qqq" )).await.unwrap();
    });


    let sender_clone = sender.clone();
    worker.spawn_local(async move {
        let sender = sender_clone;
        sender.send(( "xyz", 20, "QQQ" )).await.unwrap();
        sender.send(( "bin", 15, "boy" )).await.unwrap();
    });


    let pipe_read_xyz = MapFilterOp::new(pipe.clone(),
        |rc: Rc<RefCell<HashMap<_, _>>>| Some(rc.borrow().get("xyz").cloned()));
    worker.spawn_local(async move {
        let mut pipe_read_xyz = pipe_read_xyz;

        while let Some(val) = MoveNext::new(&mut pipe_read_xyz).await {
            println!("xyz: {:?}", val);
            sleep_yield_now().await;
            // tokio::task::yield_now().await;
        }
    });


    worker.run_until(tokio::time::sleep(std::time::Duration::from_millis(100))).await;


    let pipe_read_xyz = MapFilterOp::new(pipe.clone(),
        |rc: Rc<RefCell<HashMap<_, _>>>| Some(rc.borrow().get("xyz").cloned()));
    let mut pipe_read_xyz = pipe_read_xyz;
    println!("xyz LATE?");
    if let Some(val) = MoveNext::new(&mut pipe_read_xyz).await {
        println!("xyz LATE: {:?}", val);
        tokio::task::yield_now().await;
    }


    Ok(())
}