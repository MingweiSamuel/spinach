#![feature(never_type)]

use std::{env, process};
use std::net::SocketAddr;

use spinach::bytes::{Bytes, BytesMut};
use spinach::tokio;
use spinach::tokio::net::TcpStream;

use spinach::comp::{CompExt, DebugComp, TcpComp, TcpServerComp};
use spinach::func::Morphism;
use spinach::hide::{Hide, Qualifier};
use spinach::lattice::setunion::SetUnionRepr;
use spinach::op::{DebugOp, MorphismOp, ReadOp, TcpOp, TcpServerOp};
use spinach::tag;
use spinach::tcp_server::TcpServer;

pub struct AddrBytesFreeze;
impl Morphism for AddrBytesFreeze {
    type InLatRepr  = SetUnionRepr<tag::SINGLE, (SocketAddr, BytesMut)>;
    type OutLatRepr = SetUnionRepr<tag::SINGLE, (SocketAddr, Bytes)>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item.map_single(|(addr, bytes)| (addr, bytes.freeze()))
    }
}

pub struct StringToBytes;
impl Morphism for StringToBytes {
    type InLatRepr  = SetUnionRepr<tag::SINGLE, String>;
    type OutLatRepr = SetUnionRepr<tag::SINGLE, Bytes>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item.map_single(|string| string.into())
    }
}

/// Entry point of the application.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<!, String> {
    // Begin by parsing the arguments. We are either a server or a client, and
    // we need an address and potentially a sleep duration.
    let args: Vec<_> = env::args().collect();

    match &args[..] {
        [_, mode, url]             if mode == "server" => server(url).await?,
        [_, mode, url]             if mode == "server" => {
            client(url, tokio::io::stdin()).await?
        }
        [_, mode, url, input_file] if mode == "client" => {
            match tokio::fs::File::open(input_file).await {
                Ok(file) => client(url, file).await?,
                Err(err) => {
                    eprintln!("Failed to open inpu_file: \"{}\", error: {}", input_file, err);
                    process::exit(2);
                }
            }
        }
        _ => {
            eprintln!("Usage:\n{0} server <url>\n  or\n{0} client <url> [input_file]", args[0]);
            process::exit(1);
        }
    }
}

/// Run the server portion of the program.
async fn server(url: &str) -> Result<!, String> {

    let pool = TcpServer::bind(url).await.map_err(|e| e.to_string())?;
    let op = TcpServerOp::new(pool.clone());
    let op = DebugOp::new(op, "server");
    let op = MorphismOp::new(op, AddrBytesFreeze);
    let comp = TcpServerComp::new(op, pool);

    comp.run().await.map_err(|e| e.to_string())?;
}

/// Run the client portion of the program.
async fn client<R: tokio::io::AsyncRead + std::marker::Unpin>(url: &str, input_read: R) -> Result<!, String> {

    let (read, write) = TcpStream::connect(url).await.map_err(|e| e.to_string())?
        .into_split();

    let read_op = TcpOp::new(read);
    let read_comp = DebugComp::new(read_op);

    let write_op = ReadOp::new(input_read);
    let write_op = MorphismOp::new(write_op, StringToBytes);
    let write_comp = TcpComp::new(write_op, write);

    let result = tokio::join!(
        async {
            read_comp.run().await.map_err(|_| format!("Read failed."))
        },
        async {
            write_comp.run().await.map_err(|e| e.to_string())
        },
    );

    Err(format!("Read error: {:?}, Write error: {:?}",
        result.0.unwrap_err(), result.1.unwrap_err()))
}
