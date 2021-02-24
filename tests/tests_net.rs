use std::time::Duration;

use tokio::net::UdpSocket;

use spinach::func::*;
use spinach::op::*;

pub struct SerStrFn;
impl PureFn for SerStrFn {
    type Indomain = &'static str;
    type Outdomain = Option<Vec<u8>>;
    fn call(&self, item: Self::Indomain) -> Self::Outdomain {
        Some(item.to_owned().as_bytes().into())
    }
}

pub struct DeStrFn;
impl PureFn for DeStrFn {
    type Indomain = Vec<u8>;
    type Outdomain = Option<String>;
    fn call(&self, item: Self::Indomain) -> Self::Outdomain {
        String::from_utf8(item).ok()
    }
}

#[tokio::test]
pub async fn test_udp_echo() -> Result<(), String> {
    let sock = UdpSocket::bind("0.0.0.0:55566")
        .await
        .map_err(|err| err.to_string())?;
    sock.connect("127.0.0.1:59000")
        .await
        .map_err(|err| err.to_string())?;
    let (pull_pipe, push_pipe) = udp_op(sock);
    let comp = StaticComp::new(pull_pipe, push_pipe);

    tokio::spawn(async move {
        comp.run_moveop().await;
        panic!();
    });

    let sock = UdpSocket::bind("0.0.0.0:59000")
        .await
        .map_err(|err| err.to_string())?;
    sock.connect("127.0.0.1:55566")
        .await
        .map_err(|err| err.to_string())?;
    let (pull_pipe, mut push_pipe) = udp_op(sock);

    let recv_pipe = NullOp::new();
    let recv_pipe = DebugOp::new(recv_pipe, "RECV");
    let recv_pipe = MapFlattenMoveOp::new(recv_pipe, DeStrFn);
    let comp = StaticComp::new(pull_pipe, recv_pipe);

    tokio::spawn(async move {
        comp.run_moveop().await;
        panic!();
    });

    push_pipe.push("hello world".to_owned().into_bytes()).await;
    push_pipe
        .push("goodbye world".to_owned().into_bytes())
        .await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}
