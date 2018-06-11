extern crate chat_common;
extern crate failure;
extern crate futures;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_serde_json;

use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use structopt::StructOpt;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::timer::Delay;

use chat_common::{join_stream, messages, split_stream, Recv, Send};

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

    /// The username to connect with.
    username: String,
}

fn main() {
    let options = Options::from_args();

    let addr = SocketAddr::new(options.host, options.port);
    let client = TcpStream::connect(&addr)
        .map_err(|err| failure::Error::from(err))
        .and_then(|stream| {
            use messages::handshake::{AuthRequest, AuthResponse};

            let (recv, send) = split_stream::<AuthResponse, AuthRequest>(stream);

            Ok((recv, send))
        })
        .and_then(|(recv, send)| do_handshake(options.username, recv, send))
        .and_then(|(recv, send, username)| {
            use messages::{client::ClientMessageKind, server::ServerMessage};

            let stream = join_stream(recv, send).unwrap();
            let (recv, send) = split_stream::<ServerMessage, ClientMessageKind>(stream);

            future::ok((recv, send, username))
        })
        .and_then(|(recv, send, username)| {
            println!("authenticated as {}", username);
            let reader = read_loop(recv);

            let writer = Delay::new(Instant::now() + Duration::from_secs(5))
                .map_err(|err| failure::Error::from(err))
                .and_then(|_| {
                    use messages::client::{*, ClientMessageKind::*};

                    send.send(Goodbye(GoodbyeMessage {
                        reason: Some("timed out".into()),
                    })).map_err(|e| e.into())
                })
                .map(|_| ());

            reader.select(writer).map(|_| ()).map_err(|(err, _)| err)
        })
        .map_err(|err| {
            eprintln!("Error: {:?}", err);
            ()
        });

    tokio::run(client);
}

fn do_handshake(
    username: String,
    recv: Recv<messages::handshake::AuthResponse>,
    send: Send<messages::handshake::AuthRequest>,
) -> impl Future<
    Item = (
        Recv<messages::handshake::AuthResponse>,
        Send<messages::handshake::AuthRequest>,
        String,
    ),
    Error = failure::Error,
> {
    use messages::handshake::{AuthRequest, AuthResponse};

    send.send(AuthRequest::AuthRequest {
        username: username,
    }).map_err(|err| err.into())
        .and_then(move |send| {
            recv.into_future()
                .map_err(|(err, _)| err.into())
                .and_then(|(maybe_msg, recv)| match maybe_msg {
                    Some(AuthResponse::AuthResponse { result }) => match result {
                        Ok(username) => future::ok((recv, send, username)),
                        Err(err) => future::err(failure::err_msg(err)),
                    },

                    None => future::err(failure::err_msg("Connection closed unexpectedly.")),
                })
        })
}

fn read_loop(recv: Recv<messages::server::ServerMessage>) -> impl Future<Item = (), Error = failure::Error>
{
    use messages::server::{*, ServerMessage::*, ServerMessageKind::*};

    recv.map_err(|err| err.into())
        .for_each(|msg| {
            match msg {
                _m @ FromClient { .. } => unimplemented!(),
                FromServer(msg) => match msg {
                    Greeting(GreetingMessage { ref motd }) => {
                        println!("MOTD: {}", motd);
                    }
                },
            };

            future::ok(())
        })
        .map(|_| ())
}
