extern crate chat_common;
extern crate failure;
extern crate futures;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_serde_json;

use std::net::IpAddr;

use futures::future;
use slog::Drain;
use structopt::StructOpt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::prelude::*;
use tokio_io::codec::length_delimited;

use chat_common::*;

type Send<S> = tokio_serde_json::WriteJson<length_delimited::FramedWrite<S>, ServerMessage>;
type Recv<R> = tokio_serde_json::ReadJson<length_delimited::FramedRead<R>, ClientMessageKind>;

#[derive(Debug, StructOpt)]
#[structopt(name = "server")]
/// A simple chat server.
struct Options {
    /// The IP address to listen on.
    #[structopt(
        short = "h",
        long = "host",
        default_value = "127.0.0.1",
        env = "CHAT_HOST",
        parse(try_from_str)
    )]
    host: IpAddr,

    /// The port to bind to.
    #[structopt(short = "p", long = "port", default_value = "9999", env = "CHAT_PORT")]
    port: u16,

    /// The level of verbosity for logging. You may specify this more than once.
    #[structopt(short = "v", parse(from_occurrences))]
    verbosity: u8,
}

/// Build a drain given the terminal decorator.
fn build_drain<D>(
    decorator: D,
    min_level: slog::Level,
    max_level: slog::Level,
) -> impl Drain<Ok = Option<()>, Err = slog::Never>
where
    D: slog_term::Decorator,
{
    slog_term::FullFormat::new(decorator)
        .use_original_order()
        .use_utc_timestamp()
        .build()
        .fuse()
        .filter(move |record: &slog::Record| {
            min_level <= record.level() && record.level() <= max_level
        })
}

fn main() {
    let exit_code = {
        let options = Options::from_args();
        let log_level = match options.verbosity {
            0 => slog::Level::Info,
            1 => slog::Level::Debug,
            _ => slog::Level::Trace,
        };

        let stderr = build_drain(
            slog_term::TermDecorator::new().stderr().build(),
            slog::Level::Critical,
            slog::Level::Error,
        );
        let stdout = build_drain(
            slog_term::TermDecorator::new().stdout().build(),
            slog::Level::Warning,
            log_level,
        );

        let drain = slog::Duplicate::new(stdout, stderr).fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        let log = slog::Logger::root(drain, o!());

        info!(log, "Started server"; "options" => ?options, "version" => env!("CARGO_PKG_VERSION"));

        match run_server(log.clone(), options) {
            Ok(_) => 0,
            Err(e) => {
                crit!(log, "An nexpected error occurred"; "error" => %e);
                1
            }
        }
    };

    // `std::process::exit doesn't run any pending destructors, so we scope
    // *everything* else so that they will be dropped before here.
    std::process::exit(exit_code);
}

fn run_server(log: slog::Logger, options: Options) -> Result<(), failure::Error> {
    use std::net::SocketAddr;

    let addr = SocketAddr::new(options.host, options.port);
    let server = tokio::net::TcpListener::bind(&addr)?
        .incoming()
        .for_each({
            let log = log.clone();
            move |conn| {
                let peer_addr = conn.peer_addr()
                    .expect("Could not retrieve remote address")
                    .clone();
                let peer_addr = format!("{}", peer_addr);
                let log = log.new(o!("peer" => peer_addr));

                info!(log, "New connection.");
                tokio::spawn(handle_conn(log, conn));

                Ok(())
            }
        })
        .map_err({
            let log = log.clone();
            move |e| {
                error!(log, "Connection error."; "error" => %e);
                ()
            }
        });

    tokio::run(server);

    Ok(())
}

fn handle_conn(
    log: slog::Logger,
    stream: tokio::net::TcpStream,
) -> impl Future<Item = (), Error = ()> {
    use chat_common::*;

    let (recv, send) = stream.split();
    let send = length_delimited::FramedWrite::new(send);
    let send = tokio_serde_json::WriteJson::<_, chat_common::ServerMessage>::new(send);

    let recv = tokio_serde_json::ReadJson::<_, chat_common::ClientMessageKind>::new(
        length_delimited::FramedRead::new(recv),
    );

    do_handshake(log.clone(), send, recv)
        .map_err({
            let log = log.clone();
            move |err| {
                error!(log, "An error occurred during handshaking: {}", err);
            }
        })
        .and_then(move |(log, send, recv)| {
            send.send(ServerMessage::FromServer(ServerMessageKind::Greeting(
                GreetingMessage {
                    motd: "Hello, world!".into(),
                },
            ))).map_err(|err| failure::Error::from(err))
                .and_then({
                    let log = log.clone();
                    move |_| read_loop(log, recv)
                })
                .map_err({
                    let log = log.clone();
                    move |err| {
                        error!(log, "An unexpected error occurred: {}", err);
                    }
                })
        })
        .and_then(|_| future::ok(()))
}

fn do_handshake<S, R>(
    log: slog::Logger,
    send: Send<S>,
    recv: Recv<R>,
) -> impl Future<Item = (slog::Logger, Send<S>, Recv<R>), Error = failure::Error>
where
    S: AsyncWrite,
    R: AsyncRead,
{
    recv.into_future()
        .map_err(|(err, _)| err.into())
        .and_then(move |(maybe_msg, recv)| match maybe_msg {
            Some(ClientMessageKind::AuthRequest(AuthRequestMessage { username })) => {
                future::ok(((send, recv), username))
            }
            Some(_) => future::err(failure::err_msg("Unexpected message during handshake.")),
            None => future::err(failure::err_msg("Connection closed unexpectedly.")),
        })
        .and_then({
            let log = log.clone();
            move |((send, recv), username)| {
                let log = log.new(o!("username" => username.clone()));
                send.send(ServerMessage::FromServer(ServerMessageKind::AuthResponse(
                    AuthResponseMessage {
                        result: Ok(username),
                    },
                ))).map_err(|err| err.into())
                    .and_then(|send| {
                        info!(log, "Client authenticated.");
                        future::ok((log, send, recv))
                    })
            }
        })
}

fn read_loop<R>(log: slog::Logger, recv: Recv<R>) -> impl Future<Item = (), Error = failure::Error>
where
    R: AsyncRead,
{
    use ClientMessageKind::*;

    future::loop_fn(recv.into_future(), {
        let log = log.clone();
        move |stream_fut| {
            stream_fut
                .map_err(|(err, _)| err.into())
                .and_then(|(maybe_msg, stream)| match maybe_msg {
                    Some(msg) => future::ok((msg, stream)),
                    None => future::err(failure::err_msg("Client unexpectedly closed connection.")),
                })
                .and_then({
                    let log = log.clone();
                    move |(msg, stream)| match msg {
                        Goodbye(GoodbyeMessage { reason }) => {
                            info!(log, "Client disconnected."; "reason" => ?reason);
                            Ok(future::Loop::Break(()))
                        }
                        _ => {
                            info!(log, "Unexpected message in read loop.");
                            Ok(future::Loop::Continue(stream.into_future()))
                        }
                    }
                })
        }
    })
}
