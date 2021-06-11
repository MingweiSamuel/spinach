#![feature(never_type)]

use std::{env, process};

use tokio::net::TcpStream;

use spinach::tcp_pool::TcpPool;
use spinach::op::{DebugOp, StdinOp, TcpOp, TcpPoolOp};
use spinach::comp::{CompExt, DebugComp, TcpComp, TcpPoolComp};

/// Entry point of the application.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<!, String> {
    // Begin by parsing the arguments. We are either a server or a client, and
    // we need an address and potentially a sleep duration.
    let args: Vec<_> = env::args().collect();

    match &args[..] {
        [_, mode, url]       if mode == "server" => server(url).await?,
        [_, mode, url, name] if mode == "client" => client(url, name).await?,
        _ => {
            println!("Usage:\n{0} server <url>\n  or\n{0} client <url> <name>", args[0]);
            process::exit(1);
        }
    }
}

/// Run the server portion of the program.
async fn server(url: &str) -> Result<!, String> {

    let pool = TcpPool::bind(url).await.map_err(|e| e.to_string())?;
    let op = TcpPoolOp::<String>::new(pool.clone());
    let op = DebugOp::new(op, "server");
    let comp = TcpPoolComp::new(op, pool);

    comp.run().await.map_err(|e| e.to_string())?;
}

/// Run the client portion of the program.
async fn client(url: &str, name: &str) -> Result<!, String> {

    let (read, write) = TcpStream::connect(url).await.map_err(|e| e.to_string())?
        .into_split();

    let read_op = TcpOp::<String>::new(read);
    let read_comp = DebugComp::new(read_op);

    let write_op = StdinOp::new();
    let write_comp = TcpComp::new(write_op, write);

    #[allow(unreachable_code)]
    let result = tokio::try_join!(
        async {
            read_comp.run().await.map_err(|_| format!("Read failed for {}.", name))
        },
        async {
            write_comp.run().await.map_err(|e| e.to_string())
        },
    );

    result?;
    unreachable!();
}
