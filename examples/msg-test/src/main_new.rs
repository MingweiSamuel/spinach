//! Pair (two way radio) example.
//!
//! The pair pattern is used when there is a one-to-one peer relationship. Only one peer may be
//! connected to another peer at a time but both may speak freely.
//!
//! This example was derived from [this NNG example][1].
//!
//! [1]: https://nanomsg.org/gettingstarted/nng/pair.html
#![feature(never_type)]

use nng::{
    options::{Options, RecvTimeout},
    Error, Message, Protocol, Socket,
    Aio, AioResult,
};
use std::{env, io::Write, process, str, thread, time::Duration};

/// Entry point of the application.
pub fn main() -> Result<!, Error> {
    let args: Vec<_> = env::args().take(3).collect();

    match &args[..] {
        [_, name, url] if name == "node0" => node0(url),
        [_, name, url] => node1(url, name),
        _ => {
            println!("Usage: pipeline node0|node1 <URL> <ARG> ...");
            process::exit(1);
        }
    }
}

/// Number of outstanding requests that we can handle at a given time.
///
/// This is *NOT* the number of threads in use, but instead represents
/// outstanding work items. Select a small number to reduce memory size. (Each
/// one of these can be thought of as a request-reply loop.) Note that you will
/// probably run into limitations on the number of open file descriptors if you
/// set this too high. (If not for that limit, this could be set in the
/// thousands, each context consumes a couple of KB.)
const PARALLEL: usize = 128;

/// The listening node.
fn node0(url: &str) -> Result<!, Error> {
    // Create the socket
    let s = Socket::new(Protocol::Pair1)?;

    // Create all of the worker contexts
    let workers: Vec<_> = (0..PARALLEL)
        .map(|i| {
            let socket = s.clone();
            let aio = Aio::new(move |aio, res| worker_callback(aio, res, i, &socket))?;
            Ok(aio)
        })
        .collect::<Result<_, nng::Error>>()?;


    // Only after we have the workers do we start listening.
    s.listen(url)?;

    // Now start all of the workers listening.
    for aio in &workers {
        s.recv_async(&aio).unwrap();
    }

    println!("Workers started!");

    /// Callback function for workers.
    fn worker_callback(aio: Aio, res: AioResult, idx: usize, socket: &Socket) {
        let name = "NODE0";

        match res {
            // We successfully sent the message, wait for a new one.
            AioResult::Send(Ok(_)) => socket.recv_async(&aio).unwrap(),

            // We successfully received a message.
            AioResult::Recv(Ok(m)) => {
                let partner = str::from_utf8(&m).expect("invalid UTF-8 message");
                println!("{} AIO {}: RECEIVED \"{}\"", name, idx, partner);
                aio.sleep(Duration::from_secs(1)).unwrap();
            }

            // We successfully slept.
            AioResult::Sleep(Ok(_)) => {
                let mut msg = Message::new();
                write!(msg, "{}", name).expect("failed to write to message");
                socket.send_async(&aio, msg).unwrap();
            }

            // Anything else is an error and we will just panic.
            res => {
                panic!("Error: {:?}", res)
            }
        }
    }

    loop {
        thread::sleep(Duration::from_secs(60 * 60 * 24 * 365));
    }
}

/// The dialing node.
fn node1(url: &str, name: &str) -> Result<!, Error> {
    let s = Socket::new(Protocol::Pair1)?;
    s.dial(url)?;

    send_recv(&s, name)
}

/// Sends and receives messages on the socket.
fn send_recv(s: &Socket, name: &str) -> Result<!, Error> {
    s.set_opt::<RecvTimeout>(Some(Duration::from_millis(100)))?;
    loop {
        // Attempt to reuse the message if we can.
        let mut msg = match s.recv() {
            Ok(m) => {
                let partner = str::from_utf8(&m).expect("invalid UTF-8 message");
                println!("{}: RECEIVED \"{}\"", name, partner);

                m
            }

            Err(Error::TimedOut) => Message::new(),

            Err(e) => return Err(e),
        };

        thread::sleep(Duration::from_secs(1));

        msg.clear();
        write!(msg, "{}", name).expect("failed to write to message");

        println!("{0}: SENDING \"{0}\"", name);
        s.send(msg)?;
    }
}
