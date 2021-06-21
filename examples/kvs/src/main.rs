#![feature(core_intrinsics)]
#![feature(never_type)]

use std::{env, process};
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use spinach::tokio;
use spinach::tokio::net::TcpStream;
use spinach::bytes::{BufMut, BytesMut};

use spinach::collections::Single;
use spinach::comp::{CompExt};
use spinach::func::binary::{self, BinaryMorphism};
use spinach::func::unary::{self, Morphism};
use spinach::hide::{Hide, Qualifier};
use spinach::lattice::LatticeRepr;
use spinach::lattice::map_union::MapUnionRepr;
use spinach::lattice::set_union::SetUnionRepr;
use spinach::lattice::dom_pair::DomPairRepr;
use spinach::lattice::bottom::BottomRepr;
use spinach::lattice::ord::MaxRepr;
use spinach::lattice::pair::PairRepr;
use spinach::lattice::bytes::BytesRepr;
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

type ReqLatRepr2 = SetUnionRepr<tag::SINGLE, KvsOperation>;

type ReqLatRepr = BytesRepr<SetUnionRepr<tag::SINGLE, KvsOperation>>;
type RepLatRepr = BytesRepr<MapUnionRepr<tag::SINGLE, String, ValueLatRepr>>;

pub struct DeserializeKvsOperation;
impl Morphism for DeserializeKvsOperation {
    type InLatRepr  = ReqLatRepr;
    type OutLatRepr = SetUnionRepr<tag::OPTION, KvsOperation>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        let bytes = item.into_reveal(); // TODO REVEAL!!

        let out = match ron::de::from_bytes::<'_, (u64, <SetUnionRepr<tag::SINGLE, KvsOperation> as LatticeRepr>::Repr)>(&*bytes) {
            Ok((tid, repr)) => {
                let expected_tid = std::intrinsics::type_id::<Self::InLatRepr>();
                if expected_tid == tid {
                    Some(repr.0)
                }
                else {
                    eprintln!("Invalid TypeId, expected: {}, found: {}.", expected_tid, tid);
                    None
                }
            }
            Err(err) => {
                eprintln!("Failed to parse msg: {:?}, error: {}", bytes, err);
                None
            }
        };
        Hide::new(out)
    }
}

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
            .filter_map::<_, tag::VEC, _>(|(addr, operation)| {
                match operation {
                    KvsOperation::Read(key) => Some((key, Single(addr))),
                    KvsOperation::Write(_, _) => panic!(),
                }
            })
            .fold_into_map::<_, SetUnionRepr<tag::SINGLE, SocketAddr>, _>();

        let writes = writes
            .filter_map::<_, tag::VEC, _>(|(_addr, operation)| {
                    match operation {
                        KvsOperation::Read(_) => panic!(),
                        KvsOperation::Write(key, val) => Some((key, val)),
                    }
            })
            .fold_into_map();

        Hide::zip(reads, writes)
    }
}

pub struct CreateReplies;
impl BinaryMorphism for CreateReplies {
    type InLatReprA = SetUnionRepr<tag::HASH_SET, SocketAddr>;
    type InLatReprB = ValueLatRepr;
    type OutLatRepr = SetUnionRepr<tag::VEC, (SocketAddr, (usize, String))>;

    fn call<Y: Qualifier>(&self, item_a: Hide<Y, Self::InLatReprA>, item_b: Hide<Y, Self::InLatReprB>) -> Hide<Y, Self::OutLatRepr> {
        // TODO!!!! NOT MONOTONIC!!!!
        let mut out = Vec::new();
        for addr in item_a.into_reveal() { // TODO!! REVEAL!!!
            out.push((addr, item_b.reveal_ref().clone()));
        }
        Hide::new(out)
    }
}

pub struct SerializeValues;
impl Morphism for SerializeValues {
    type InLatRepr  = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::VEC, (SocketAddr, (usize, String))>>;
    type OutLatRepr = MapUnionRepr<tag::VEC, SocketAddr, RepLatRepr>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        let mut out = Vec::new();
        for (k, vals) in item.into_reveal() { // TODO! REVEAL!
            for (addr, val) in vals {
                let tid = std::intrinsics::type_id::<RepLatRepr>();
                let item = (tid, (&*k, val));

                let mut writer = BytesMut::new().writer();
                ron::ser::to_writer(&mut writer, &item).expect("Failed to serialize");
                let bytes = writer.into_inner().freeze();

                out.push((addr, bytes));
            }
        }
        Hide::new(out)
    }
}

/// Run the server portion of the program.
async fn server(url: &str) -> Result<!, String> {

    let server = TcpServer::bind(url).await.map_err(|e| e.to_string())?;
    // let (op_reads, op_writes) = TcpServerOp::<ReqLatRepr>::new(server.clone())
    TcpServerOp::<ReqLatRepr>::new(server.clone())
        // .debug("ingress")
        // .morphism(unary::HashPartitioned::new(DeserializeKvsOperation))
        .morphism(unary::HashPartitioned::new(unary::Deserialize::<ReqLatRepr2>::new()))
        // .debottom()
        .morphism_closure(|item| item.flatten_keyed::<tag::VEC>())
        // .morphism(Switch)
        // // .debug("split")
        // .switch();

    // type ReadsLatRepr = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::HASH_SET, SocketAddr>>;
    // let op_reads = op_reads
    //     // .debug("read")
    //     .lattice_default::<ReadsLatRepr>();

    // type WritesLatRepr = MapUnionRepr<tag::HASH_MAP, String, ValueLatRepr>;
    // let op_writes = op_writes
    //     // .debug("write")
    //     .lattice_default::<WritesLatRepr>();

    // let binary_func = binary::HashPartitioned::<String, _>::new(CreateReplies);
    // let comp = op_reads
    //     .binary(op_writes, binary_func)
    //     // .debug("after binop")
    //     .morphism(SerializeValues)
    //     .comp_tcp_server(server);

    // comp
    //     .run()
    //     .await
    //     .map_err(|e| format!("TcpComp error: {:?}", e))?;

    ;unreachable!();
}



pub struct DeserializeValue;
impl Morphism for DeserializeValue {
    type InLatRepr  = RepLatRepr;
    type OutLatRepr = MapUnionRepr::<tag::OPTION, String, ValueLatRepr>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        let bytes = item.into_reveal(); // TODO REVEAL!!

        let out = match ron::de::from_bytes::<'_, (u64, <MapUnionRepr<tag::SINGLE, String, ValueLatRepr> as LatticeRepr>::Repr)>(&*bytes) {
            Ok((tid, repr)) => {
                let expected_tid = std::intrinsics::type_id::<Self::InLatRepr>();
                if expected_tid == tid {
                    Some(repr.0)
                }
                else {
                    eprintln!("Invalid TypeId, expected: {}, found: {}.", expected_tid, tid);
                    None
                }
            }
            Err(err) => {
                eprintln!("Failed to parse msg: {:?}, error: {}", bytes, err);
                None
            }
        };

        Hide::new(out)
    }
}

pub struct SerializeKvsOperation;
impl Morphism for SerializeKvsOperation {
    type InLatRepr  = SetUnionRepr<tag::SINGLE, String>;
    type OutLatRepr = BottomRepr<ReqLatRepr>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        // TODO! REVEAL!!
        match ron::de::from_str::<KvsOperation>(&item.reveal_ref().0) {
            Ok(operation) => {
                let tid = std::intrinsics::type_id::<Self::OutLatRepr>();
                let item = (tid, operation);

                let mut writer = BytesMut::new().writer();
                ron::ser::to_writer(&mut writer, &item).expect("Failed to serialize");
                let bytes = writer.into_inner().freeze();

                Hide::new(Some(bytes))
            },
            Err(err) => {
                eprintln!("Failed to parse operation, error: {}", err);
                Hide::new(None)
            }
        }
    }
}

/// Run the client portion of the program.
async fn client<R: tokio::io::AsyncRead + std::marker::Unpin>(url: &str, input_read: R) -> Result<!, String> {

    let (read, write) = TcpStream::connect(url).await.map_err(|e| e.to_string())?
        .into_split();

    let read_comp = TcpOp::new(read)
        .morphism(DeserializeValue)
        .comp_debug("read");

    let write_comp = ReadOp::new(input_read)
        .morphism(SerializeKvsOperation)
        .debottom()
        .comp_tcp(write);

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
