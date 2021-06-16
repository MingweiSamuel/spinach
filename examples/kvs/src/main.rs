#![feature(never_type)]

use std::{env, process};
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use spinach::tokio;
use spinach::tokio::net::TcpStream;
use spinach::bytes::{BufMut, Bytes, BytesMut};

use spinach::comp::{CompExt};
use spinach::func::binary::{BinaryMorphism, HashPartitioned};
use spinach::func::unary::Morphism;
use spinach::hide::{Hide, Qualifier};
use spinach::lattice::LatticeRepr;
use spinach::lattice::mapunion::MapUnionRepr;
use spinach::lattice::setunion::SetUnionRepr;
use spinach::lattice::dompair::DomPairRepr;
use spinach::lattice::ord::MaxRepr;
use spinach::op::{OpExt, ReadOp, TcpOp, TcpServerOp};
use spinach::tag;
use spinach::tcp_server::TcpServer;

type ValueLatRepr = DomPairRepr<MaxRepr<usize>, MaxRepr<String>>;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KvsOperation {
    Read(String),
    Write(String, <ValueLatRepr as LatticeRepr>::Repr),
}

pub struct DeserializeKvsOperation;
impl Morphism for DeserializeKvsOperation {
    type InLatRepr  = SetUnionRepr<tag::SINGLE, (SocketAddr, BytesMut)>;
    type OutLatRepr = SetUnionRepr<tag::OPTION, (SocketAddr, KvsOperation)>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item
            .filter_map_single(|(addr, bytes)| {
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
    type InLatRepr  = SetUnionRepr<tag::OPTION, (SocketAddr, KvsOperation)>;
    type OutLatRepr = MapUnionRepr<tag::VEC, String, SetUnionRepr<tag::SINGLE, SocketAddr>>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item
            .filter_map::<_, tag::VEC, _>(|(addr, operation)| {
                match operation {
                    KvsOperation::Read(key) => Some((key, addr)),
                    KvsOperation::Write(_, _) => None,
                }
            })
            .into_map()
    }
}

pub struct SplitWrites;
impl Morphism for SplitWrites {
    type InLatRepr  = SetUnionRepr<tag::OPTION, (SocketAddr, KvsOperation)>;
    type OutLatRepr = MapUnionRepr<tag::OPTION, String, ValueLatRepr>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        // TODO!!! REVEAL!!!!
        let out = item.into_reveal()
            .and_then(|(_addr, operation)| {
                match operation {
                    KvsOperation::Read(_) => None,
                    KvsOperation::Write(key, val) => {
                        Some((key, val))
                    }
                }
            });
        Hide::new(out)
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

pub struct CreateReplies;
impl BinaryMorphism for CreateReplies {
    type InLatReprA = SetUnionRepr<tag::HASH_SET, SocketAddr>;
    type InLatReprB = ValueLatRepr;
    type OutLatRepr = SetUnionRepr<tag::VEC, (SocketAddr, (usize, String))>;
    fn call<Y: Qualifier>(&self, item_a: Hide<Y, Self::InLatReprA>, item_b: Hide<Y, Self::InLatReprB>) -> Hide<Y, Self::OutLatRepr> {
        let mut out = Vec::new();
        for addr in item_a.into_reveal() { // TODO!! REVEAL!!!
            out.push((addr, item_b.reveal_ref().clone()));
        }
        Hide::new(out)
    }
}

pub struct ServerSerialize;
impl Morphism for ServerSerialize {
    type InLatRepr  = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::VEC, (SocketAddr, (usize, String))>>;
    type OutLatRepr = SetUnionRepr<tag::VEC, (SocketAddr, Bytes)>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        let mut out = Vec::new();

        for (k, vals) in item.into_reveal() { // TODO! REVEAL!
            for (addr, val) in vals {
                let item = (&*k, val);

                let mut writer = BytesMut::new().writer();
                ron::ser::to_writer(&mut writer, &item).expect("Failed to serialize");
                let bytes = writer.into_inner().freeze();

                out.push((addr, bytes));
            }
        }

        Hide::new(out)

        // item
        //     .map(|(key, addr, value)| {
        //         let item = (key, value);

        //         let mut writer = BytesMut::new().writer();
        //         ron::ser::to_writer(&mut writer, &item).expect("Failed to serialize");
        //         let bytes = writer.into_inner().freeze();

        //         (addr, bytes)
        //     })
    }
}

/// Run the server portion of the program.
async fn server(url: &str) -> Result<!, String> {

    let pool = TcpServer::bind(url).await.map_err(|e| e.to_string())?;
    let [op_reads, op_writes] = TcpServerOp::new(pool.clone())
        // .debug("ingress")
        .morphism(DeserializeKvsOperation)
        .fixed_split();

    type ReadsLatRepr = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::HASH_SET, SocketAddr>>;
    let op_reads = op_reads
        .morphism(SplitReads)
        .lattice_default::<ReadsLatRepr>();

    type WritesLatRepr = MapUnionRepr<tag::HASH_MAP, String, ValueLatRepr>;
    let op_writes = op_writes
        .morphism(SplitWrites)
        .lattice_default::<WritesLatRepr>();

    let binary_func = HashPartitioned::new(CreateReplies);
    op_reads
        .binary(op_writes, binary_func)
        .morphism(ServerSerialize)
        .comp_tcp_server(pool)
        .run()
        .await
        .map_err(|e| format!("TcpComp error: {:?}", e))?;
}

/// Run the client portion of the program.
async fn client<R: tokio::io::AsyncRead + std::marker::Unpin>(url: &str, input_read: R) -> Result<!, String> {

    let (read, write) = TcpStream::connect(url).await.map_err(|e| e.to_string())?
        .into_split();

    let read_comp = TcpOp::new(read)
        .morphism(BytesToString)
        .comp_null();

    let write_comp = ReadOp::new(input_read)
        .morphism(StringToBytes)
        .comp_tcp(write);

    let result = tokio::try_join!(
        async {
            read_comp.run().await.map_err(|_| format!("Read failed."))
        },
        async {
            let err = write_comp.run().await.map_err(|e| e.to_string());
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            err
        },
    );

    result?;
    unreachable!();

    // Err(format!("Read error: {:?}, Write error: {:?}",
    //     result.0.unwrap_err(), result.1.unwrap_err()))
}

/// Entry point of the application.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<!, String> {
    // Begin by parsing the arguments. We are either a server or a client, and
    // we need an address and potentially a sleep duration.
    let args: Vec<_> = env::args().collect();

    match &args[..] {
        [_, mode, url]             if mode == "server" => server(url).await?,
        [_, mode, url]             if mode == "client" => {
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