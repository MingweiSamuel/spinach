#![feature(never_type)]

use std::{env, process};

// use spinach::bytes::Bytes;
use spinach::tokio;
// use spinach::tokio::net::TcpStream;

// use spinach::comp::{CompExt};
// use spinach::hide::Hide;
// use spinach::lattice::set_union::SetUnionRepr;
// use spinach::lattice::bytes::BytesRepr;
// use spinach::op::{OpExt, ReadOp, TcpOp, TcpServerOp};
// use spinach::tcp_server::TcpServer;
// use spinach::tag;

// type WireLatRepr = BytesRepr<SetUnionRepr<tag::SINGLE, String>>;

// /// Run the server portion of the program.
// async fn server(url: &str) -> Result<!, String> {

//     let server = TcpServer::bind(url).await.map_err(|e| e.to_string())?;

//     // SetUnion<(SocketAddr, Bytes)>
//     TcpServerOp::<WireLatRepr>::new(server.clone())
//         .debug("server")
//         .morphism_closure(|item| Hide::new(item.into_reveal().0.try_into().unwrap()))
//         .morphism_closure(|item| item.map_one(|(addr, bytes)| (addr, bytes.freeze())))
//         // TODO CONVERT: MapUnion<SocketAddr, SetUnion<Bytes>>
//         .comp_tcp_server(server)
//         .run()
//         .await
//         .map_err(|e| e.to_string())?;
// }

// /// Run the client portion of the program.
// async fn client<R: tokio::io::AsyncRead + std::marker::Unpin>(url: &str, input_read: R) -> Result<!, String> {

//     let (read, write) = TcpStream::connect(url).await.map_err(|e| e.to_string())?
//         .into_split();

//     let read_comp = TcpOp::<WireLatRepr>::new(read)
//         .comp_debug("read");

//     let write_comp = ReadOp::new(input_read)
//         .morphism_closure(|item| item.map_one::<Bytes, _>(|string| string.into()))
//         .morphism_closure(|item| Hide::<_, WireLatRepr>::new(item.into_reveal().0)) // TODO reveal.
//         .comp_tcp(write);

//     let result = tokio::join!(
//         async {
//             read_comp.run().await.map_err(|_| format!("Read failed."))
//         },
//         async {
//             write_comp.run().await.map_err(|e| e.to_string())
//         },
//     );

//     Err(format!("Read error: {:?}, Write error: {:?}",
//         result.0.unwrap_err(), result.1.unwrap_err()))
// }

/// Entry point of the application.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<!, String> {
    // Begin by parsing the arguments. We are either a server or a client, and
    // we need an address and potentially a sleep duration.
    let args: Vec<_> = env::args().collect();

    match &*args {
        // [_, mode, url]             if mode == "server" => server(url).await?,
        // [_, mode, url]             if mode == "client" => client(url, tokio::io::stdin()).await?,
        // [_, mode, url, input_file] if mode == "client" => {
        //     match tokio::fs::File::open(input_file).await {
        //         Ok(file) => client(url, file).await?,
        //         Err(err) => {
        //             eprintln!("Failed to open input_file: \"{}\", error: {}", input_file, err);
        //             process::exit(2);
        //         }
        //     }
        // }
        _ => {
            eprintln!("Usage:\n{0} server <url>\n  or\n{0} client <url> [input_file]", args[0]);
            process::exit(1);
        }
    }
}