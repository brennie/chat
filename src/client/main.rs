extern crate chat_common;
extern crate futures;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_serde_json;

use chat_common::*;
use std::net::{IpAddr, SocketAddr};
use structopt::StructOpt;
use tokio::prelude::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "server")]
/// A simple chat server.
struct Options {
    /// The IP address to listen on.
    #[structopt(name = "HOST", parse(try_from_str))]
    host: IpAddr,

    /// The port to bind to.
    #[structopt(short = "p", long = "port", default_value = "9999")]
    port: u16,
}

fn main() {
    let options = Options::from_args();

    let addr = SocketAddr::new(options.host, options.port);
    let client = tokio::net::TcpStream::connect(&addr)
        .map_err(|err| {
            eprintln!("Connection error: {:?}", err);
            ()
        })
        .and_then(|stream| {
            let (send, recv) = tokio_io::codec::length_delimited::Framed::new(stream).split();

            let send = tokio_serde_json::WriteJson::<_, chat_common::ClientMessageKind>::new(send);
            let recv = tokio_serde_json::ReadJson::<_, chat_common::ServerMessage>::new(recv);

            recv.into_future()
                .map_err(|(err, _)| eprintln!("Stream error: {:?}", err))
                .and_then(|(maybe_msg, recv)| {
                    match maybe_msg {
                        Some(ServerMessage::FromServer(ServerMessageKind::Greeting(
                            GreetingMessage { ref motd },
                        ))) => {
                            println!("recvd: {}", motd);
                            futures::future::ok((send, recv))
                        }

                        Some(_) => {
                            eprintln!("Unexpected msg");
                            futures::future::err(())
                        }

                        None => {
                            eprintln!("Server closed connection?");
                            futures::future::err(())
                        }
                    }.and_then(|(send, recv)| {
                        send.send(ClientMessageKind::Goodbye(GoodbyeMessage {
                            reason: Some("Transaction complete".into()),
                        })).map_err(|err| eprintln!("Stream error: {:?}", err))
                            .map(move |send| (send, recv))
                    })
                        .and_then(|(send, recv)| {
                            let send = send.into_inner();
                            let recv = recv.into_inner();
                            let stream = send.reunite(recv)
                                .expect("send and recv not matched pair")
                                .into_inner();

                            stream
                                .shutdown(std::net::Shutdown::Both)
                                .expect("Could not close peer connection");

                            futures::future::ok(())
                        })
                })
        });

    tokio::run(client);
}
