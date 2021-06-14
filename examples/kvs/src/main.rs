#![feature(never_type)]

use std::{env, process};
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use spinach::tokio;
use spinach::tokio::net::TcpStream;
use spinach::bytes::{BufMut, Bytes, BytesMut};

use spinach::comp::{CompExt, DebugComp, TcpComp, TcpServerComp};
use spinach::func::Morphism;
use spinach::hide::{Hide, Qualifier};
use spinach::lattice::setunion::SetUnionRepr;
use spinach::op::{DebugOp, MorphismOp, Splitter, ReadOp, SymHashJoinOp, TcpOp, TcpServerOp};
use spinach::tag;
use spinach::tcp_server::TcpServer;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KvsOperation {
    Read(String),
    Write(String, String),
}

pub struct DeserializeKvsOperation;
impl Morphism for DeserializeKvsOperation {
    type InLatRepr  = SetUnionRepr<tag::SINGLE, (SocketAddr, BytesMut)>;
    type OutLatRepr = SetUnionRepr<tag::VEC, (SocketAddr, KvsOperation)>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item
            .filter_map(|(addr, bytes)| {
                match ron::de::from_bytes(&*bytes) {
                    Ok(string) => Some((addr, string)),
                    Err(err) => {
                        println!("Failed to parse msg: {}", err);
                        None
                    }
                }
            })
    }
}

pub struct SplitReads;
impl Morphism for SplitReads {
    type InLatRepr  = SetUnionRepr<tag::VEC, (SocketAddr, KvsOperation)>;
    type OutLatRepr = SetUnionRepr<tag::VEC, (String, SocketAddr)>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item
            .filter_map(|(addr, operation)| {
                match operation {
                    KvsOperation::Read(key) => Some((key, addr)),
                    KvsOperation::Write(_, _) => None,
                }
            })
    }
}

pub struct SplitWrites;
impl Morphism for SplitWrites {
    type InLatRepr  = SetUnionRepr<tag::VEC, (SocketAddr, KvsOperation)>;
    type OutLatRepr = SetUnionRepr<tag::VEC, (String, String)>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item
            .filter_map(|(_addr, operation)| {
                match operation {
                    KvsOperation::Read(_) => None,
                    KvsOperation::Write(key, val) => Some((key, val)),
                }
            })
    }
}

pub struct ServerSerialize;
impl Morphism for ServerSerialize {
    type InLatRepr  = SetUnionRepr<tag::VEC, (String, SocketAddr, String)>;
    type OutLatRepr = SetUnionRepr<tag::VEC, (SocketAddr, Bytes)>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item
            .map(|(key, addr, value)| {
                let item = (key, value);

                let mut writer = BytesMut::new().writer();
                ron::ser::to_writer(&mut writer, &item).expect("Failed to serialize");
                let bytes = writer.into_inner().freeze();

                (addr, bytes)
            })
    }
}

pub struct BytesToString;
impl Morphism for BytesToString {
    type InLatRepr  = SetUnionRepr<tag::SINGLE, BytesMut>;
    type OutLatRepr = SetUnionRepr<tag::OPTION, String>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item.filter_map_single(|bytes| {
            match String::from_utf8(bytes.to_vec()) {
                Ok(string) => Some(string),
                Err(err) => {
                    eprintln!("Failed to parse bytes as utf8 string. Error: {}, bytes: {:?}", err, bytes);
                    None
                }
            }
        })
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
    let op = DebugOp::new(op, "ingress");
    let op = MorphismOp::new(op, DeserializeKvsOperation);

    let splitter = Splitter::new(op);

    let op_reads = MorphismOp::new(splitter.add_split(), SplitReads);
    let op_writes = MorphismOp::new(splitter.add_split(), SplitWrites);
    let op = SymHashJoinOp::new(op_reads, op_writes);

    let op = MorphismOp::new(op, ServerSerialize);

    let comp = TcpServerComp::new(op, pool);
    comp.run().await.map_err(|e| e.to_string())?;
}

/// Run the client portion of the program.
async fn client<R: tokio::io::AsyncRead + std::marker::Unpin>(url: &str, input_read: R) -> Result<!, String> {

    let (read, write) = TcpStream::connect(url).await.map_err(|e| e.to_string())?
        .into_split();

    let read_op = TcpOp::new(read);
    let read_op = MorphismOp::new(read_op, BytesToString);
    let read_comp = DebugComp::new(read_op);

    let write_op = ReadOp::new(input_read);
    let write_op = MorphismOp::new(write_op, StringToBytes);
    let write_comp = TcpComp::new(write_op, write);

    #[allow(unreachable_code)]
    let result = tokio::try_join!(
        async {
            read_comp.run().await.map_err(|_| format!("Read failed."))
        },
        async {
            write_comp.run().await.map_err(|e| e.to_string())
        },
    );

    result?;
    unreachable!();
}
