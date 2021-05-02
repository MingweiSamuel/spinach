use std::collections::BTreeMap;
use std::time::Duration;

use tokio::net::UdpSocket;

use spinach::lattice::*;
use spinach::comp::*;
use spinach::op::*;
use spinach::func::*;

#[tokio::test]
pub async fn test_chat() -> Result<(), String> {
    
    // let sock = UdpSocket::bind("0.0.0.0:44400")
    //     .await
    //     .map_err(|err| err.to_string())?;
    // sock.connect("127.0.0.1:44401")
    //     .await
    //     .map_err(|err| err.to_string())?;
    // let (pull_pipe, push_pipe) = udp_op::<String>(sock);

    let (push_pipe, pull_pipe) = channel_op::<Vec<u8>>(4);
    
    pub struct StringMapFn;
    impl PureFn for StringMapFn {
        type Indomain = Vec<u8>;
        type Outdomain = Option<BTreeMap<String, String>>;
        fn call(&self, item: Self::Indomain) -> Self::Outdomain {
            let item = String::from_utf8(item).ok()?;
            let mut lines: Vec<&str> = item.lines().take(2).collect();
            if 2 != lines.len() {
                return None
            }
            let mut out = BTreeMap::<String, String>::new();
            let val = lines.pop().unwrap().to_owned();
            out.insert(
                lines.pop().unwrap().to_owned(),
                val);
            Some(out)
        }
    }
    // let pull_pipe = ToOwnedOp::new(pull_pipe);
    let pull_pipe = MapFilterMoveOp::new(pull_pipe, StringMapFn);

    type MyLattice = MapUnion<BTreeMap<String, Max<String>>>;

    let mut comp = LatticeComp::<_, _, MyLattice>::new(pull_pipe, BTreeMap::new());

    let mut push_pipe = push_pipe;
    push_pipe.push("0001\nMingwei: Hello Joe.".as_bytes().to_owned()).await;
    push_pipe.push("0002\nJoe: Hi.".as_bytes().to_owned()).await;
    push_pipe.push("0003\nMatthew: Goodybye.".as_bytes().to_owned()).await;

    comp.tick().await;
    comp.tick().await;

    comp.add_split(CloneOp::new(DebugOp::new(NullOp::new(), "x"))).await;

    tokio::spawn(async move {
        comp.run().await;
        panic!();
    });

    push_pipe.push("0004\nPranav: akasjklfdkl.".as_bytes().to_owned()).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}
