//! A simple asynchronous RPC server example.
//!
//! This example shows how to write an asynchronous RPC server using Contexts.
//! This is heavily based on the `nng` demonstration program of the same name.
//!
//! The protocol is simple: the client sends a request with the number of
//! milliseconds to wait, the server waits that long and sends back an empty
//! reply.
use nng::{Aio, AioResult, Context, Message, Protocol, Socket};
use std::{
    env, process, thread,
    time::{Duration, Instant},
    io::Write,
};

/// Number of outstanding requests that we can handle at a given time.
///
/// This is *NOT* the number of threads in use, but instead represents
/// outstanding work items. Select a small number to reduce memory size. (Each
/// one of these can be thought of as a request-reply loop.) Note that you will
/// probably run into limitations on the number of open file descriptors if you
/// set this too high. (If not for that limit, this could be set in the
/// thousands, each context consumes a couple of KB.)
const PARALLEL: usize = 128;

/// Entry point of the application.
fn main() -> Result<(), nng::Error> {
    // Begin by parsing the arguments. We are either a server or a client, and
    // we need an address and potentially a sleep duration.
    let args: Vec<_> = env::args().collect();

    match &args[..] {
        [_, t, url] if t == "server" => server(url),
        [_, t, url, name] if t == "client" => client(url, name),
        _ => {
            println!("Usage:\nasync server <url>\n  or\nasync client <url> <ms>");
            process::exit(1);
        }
    }
}

/// Run the client portion of the program.
fn client(url: &str, name: &str) -> Result<(), nng::Error> {
    let s = Socket::new(Protocol::Rep0)?;
    s.dial(url)?;

    loop {
        let start = Instant::now();
        let msg = s.recv()?;
        let msg_str = std::str::from_utf8(&msg).expect("invalid UTF-8 message");
        println!("{:?}", msg_str);

        std::thread::sleep(Duration::from_millis(1000));

        let mut msg = Message::new();
        write!(msg, "Hello from {}", name).expect("failed to write to message");
        s.send(msg)?;

        let dur = Instant::now().duration_since(start);
        let subsecs: u64 = dur.subsec_millis().into();
        println!(
            "Request took {} milliseconds",
            dur.as_secs() * 1000 + subsecs
        );
    }

    // Ok(())
}

/// Run the server portion of the program.
fn server(url: &str) -> Result<(), nng::Error> {
    // Create the socket
    let s = Socket::new(Protocol::Req0)?;

    // Create all of the worker contexts
    let workers: Vec<_> = (0..PARALLEL)
        .map(|idx| {
            let ctx = Context::new(&s)?;
            let ctx_clone = ctx.clone();
            let aio = Aio::new(move |aio, res| worker_callback(aio, res, idx, &ctx_clone))?;
            Ok((aio, ctx))
        })
        .collect::<Result<_, nng::Error>>()?;

    // Only after we have the workers do we start listening.
    s.listen(url)?;

    // Now start all of the workers listening.
    for (a, c) in &workers {
        let mut msg = Message::new();
        write!(msg, "Hello World").expect("failed to write to message");
        c.send(a, msg)?;
    }

    println!("Workers started!");

    thread::sleep(Duration::from_secs(60 * 60 * 24 * 365));

    Ok(())
}

/// Callback function for workers.
fn worker_callback(aio: Aio, res: AioResult, idx: usize, ctx: &Context) {
    match res {
        // We successfully sent the message.
        AioResult::Send(Ok(_)) => {
            // Wait for the response.
            ctx.recv(&aio).unwrap()
        }

        // We successfully received a response.
        AioResult::Recv(Ok(msg)) => {
            let msg_str = std::str::from_utf8(&msg).expect("invalid UTF-8 message");
            println!("AIO {}: {}", idx, msg_str);
            aio.sleep(Duration::from_millis(1)).unwrap();
        }

        // We successfully slept.
        AioResult::Sleep(Ok(_)) => {
            // Send a new msg.
            let mut msg = Message::new();
            write!(msg, "Hello World 2").expect("failed to write to message");
            ctx.send(&aio, msg).unwrap();
        }

        // Anything else is an error and we will just panic.
        res => {
            panic!("Error: {:?}", res)
        }
    }
}
