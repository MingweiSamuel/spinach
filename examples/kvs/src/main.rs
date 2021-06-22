#![feature(core_intrinsics)]
#![feature(never_type)]

use std::{env, process};
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use spinach::tokio;
use spinach::tokio::net::TcpStream;

use spinach::collections::Single;
use spinach::comp::{CompExt};
use spinach::func::binary::{HashPartitioned, TableProduct};
use spinach::func::unary::{Morphism};
use spinach::hide::{Hide, Qualifier};
use spinach::lattice::LatticeRepr;
use spinach::lattice::map_union::MapUnionRepr;
use spinach::lattice::set_union::SetUnionRepr;
use spinach::lattice::dom_pair::DomPairRepr;
use spinach::lattice::ord::MaxRepr;
use spinach::lattice::pair::PairRepr;
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

type RequestLatRepr = SetUnionRepr<tag::SINGLE, KvsOperation>;
type ResponseLatRepr = MapUnionRepr<tag::VEC, String, ValueLatRepr>;

pub struct Switch;
impl Morphism for Switch {
    type InLatRepr  = SetUnionRepr<tag::VEC, (SocketAddr, KvsOperation)>;
    type OutLatRepr = PairRepr<
        MapUnionRepr<tag::VEC, String, SetUnionRepr<tag::VEC, SocketAddr>>,
        MapUnionRepr<tag::VEC, String, ValueLatRepr>,
    >;

    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        let (reads, writes) = item.switch::<tag::VEC, _>(|(_addr, operation)| {
            match operation {
                KvsOperation::Read(_) => true,
                KvsOperation::Write(_, _) => false,
            }
        });

        let reads = reads
            .map::<_, tag::VEC, _>(|(addr, operation)| {
                match operation {
                    KvsOperation::Read(key) => Single((key, Single(addr))),
                    KvsOperation::Write(_, _) => panic!(),
                }
            })
            .fold::<MapUnionRepr<tag::VEC, String, SetUnionRepr<tag::VEC, SocketAddr>>, MapUnionRepr<tag::SINGLE, String, SetUnionRepr<tag::SINGLE, SocketAddr>>>();

        let writes = writes
            .map::<_, tag::VEC, _>(|(_addr, operation)| {
                    match operation {
                        KvsOperation::Read(_) => panic!(),
                        KvsOperation::Write(key, val) => Single((key, val)),
                    }
            })
            .fold::<MapUnionRepr<tag::VEC, String, ValueLatRepr>, MapUnionRepr<tag::SINGLE, String, ValueLatRepr>>();

        Hide::zip(reads, writes)
    }
}

/// Run the server portion of the program.
async fn server(url: &str) -> Result<!, String> {

    let server = TcpServer::bind(url).await.map_err(|e| e.to_string())?;
    let (op_reads, op_writes) = TcpServerOp::<RequestLatRepr>::new(server.clone())
        // .debug("ingress")
        .morphism_closure(|item| item.flatten_keyed::<tag::VEC>())
        .morphism(Switch)
        // .debug("split")
        .switch();

    type ReadsLatRepr = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::HASH_SET, SocketAddr>>;
    let op_reads = op_reads
        // .debug("read")
        .lattice_default::<ReadsLatRepr>();

    type WritesLatRepr = MapUnionRepr<tag::HASH_MAP, String, ValueLatRepr>;
    let op_writes = op_writes
        // .debug("write")
        .lattice_default::<WritesLatRepr>();

    let binary_func = HashPartitioned::<String, _>::new(
        TableProduct::<_, _, _, MapUnionRepr<tag::VEC, _, _>>::new());

    let comp = op_reads
        .binary(op_writes, binary_func)
        // .debug("after binop")
        .morphism_closure(|item| item.transpose::<tag::VEC, tag::VEC>())
        .comp_tcp_server::<ResponseLatRepr, _>(server);

    comp
        .run()
        .await
        .map_err(|e| format!("TcpComp error: {:?}", e))?;
}


pub struct ParseKvsOperation;
impl Morphism for ParseKvsOperation {
    type InLatRepr  = SetUnionRepr<tag::SINGLE, String>;
    type OutLatRepr = SetUnionRepr<tag::OPTION, KvsOperation>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item.filter_map_one(|input| {
            match ron::de::from_str::<KvsOperation>(&*input) {
                Ok(operation) => {
                    Some(operation)
                },
                Err(err) => {
                    eprintln!("Failed to parse operation, error: {}", err);
                    None
                }
            }
        })
    }
}

/// Run the client portion of the program.
async fn client<R: tokio::io::AsyncRead + std::marker::Unpin>(url: &str, input_read: R) -> Result<!, String> {

    let (read, write) = TcpStream::connect(url).await.map_err(|e| e.to_string())?
        .into_split();

    let read_comp = TcpOp::<ResponseLatRepr>::new(read)
        .comp_debug("read");

    let write_comp = ReadOp::new(input_read)
        .morphism(ParseKvsOperation)
        .debottom()
        .comp_tcp::<RequestLatRepr>(write);

    #[allow(unreachable_code)]
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

    match &*args {
        [_, mode, url]             if mode == "server" => server(url).await?,
        [_, mode, url]             if mode == "client" => client(url, tokio::io::stdin()).await?,
        [_, mode, url, input_file] if mode == "client" => {
            match tokio::fs::File::open(input_file).await {
                Ok(file) => client(url, file).await?,
                Err(err) => {
                    eprintln!("Failed to open input_file: \"{}\", error: {}", input_file, err);
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
