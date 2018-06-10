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
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::timer::Delay;
use tokio_io::codec::length_delimited;

use chat_common::*;

type Send<S> = tokio_serde_json::WriteJson<length_delimited::FramedWrite<S>, ClientMessageKind>;
type Recv<R> = tokio_serde_json::ReadJson<length_delimited::FramedRead<R>, ServerMessage>;

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
            let (recv, send) = stream.split();
            let send = length_delimited::FramedWrite::new(send);
            let send = tokio_serde_json::WriteJson::<_, chat_common::ClientMessageKind>::new(send);

            let recv = length_delimited::FramedRead::new(recv);
            let recv = tokio_serde_json::ReadJson::<_, chat_common::ServerMessage>::new(recv);

            Ok((send, recv))
        })
        .and_then(|(send, recv)| do_handshake(options.username, send, recv))
        .and_then(|((send, recv), username)| {
            let reader = read_loop(recv);

            let writer = Delay::new(Instant::now() + Duration::from_secs(5))
                .map_err(|err| failure::Error::from(err))
                .and_then(|_| {
                    use ClientMessageKind::*;

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

 fn do_handshake<S, R>(
    username: String,
    send: Send<S>,
    recv: Recv<R>,
) -> impl Future<Item = ((Send<S>, Recv<R>), String), Error = failure::Error>
where
    S: AsyncWrite,
    R: AsyncRead,
{
    send.send(ClientMessageKind::AuthRequest(AuthRequestMessage {
        username: username.into(),
    })).map_err(|err| err.into())
        .and_then(move |send| {
            recv.into_future()
                .map_err(|(err, _)| err.into())
                .and_then(|(maybe_msg, recv)| match maybe_msg {
                    Some(ServerMessage::FromServer(ServerMessageKind::AuthResponse(
                        AuthResponseMessage { result },
                    ))) => match result {
                        Ok(username) => future::ok(((send, recv), username)),
                        Err(err) => future::err(failure::err_msg(err)),
                    },

                    Some(_) => {
                        future::err(failure::err_msg("Unexpected message during handshake."))
                    }

                    None => future::err(failure::err_msg("Connection closed unexpectedly.")),
                })
        })
}

fn read_loop<R>(recv: Recv<R>) -> impl Future<Item = (), Error = failure::Error>
where
    R: AsyncRead,
{
    recv.map_err(|err| err.into())
        .for_each(|msg| {
            use ServerMessage::*;
            use ServerMessageKind::*;

            match msg {
                _m @ FromClient { .. } => unimplemented!(),
                FromServer(msg) => match msg {
                    Greeting(GreetingMessage { ref motd }) => {
                        println!("MOTD: {}", motd);
                    }
                    _ => unimplemented!(),
                },
            };

            future::ok(())
        })
        .map(|_| ())
}
