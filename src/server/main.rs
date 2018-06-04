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

use slog::Drain;
use std::net::IpAddr;
use structopt::StructOpt;
use tokio::prelude::*;

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
    use chat_common::{GoodbyeMessage, GreetingMessage, MessageKind::*};
    let (send, recv) = tokio_io::codec::length_delimited::Framed::new(stream).split();

    let send = tokio_serde_json::WriteJson::<_, chat_common::MessageKind>::new(send);
    let recv = tokio_serde_json::ReadJson::<_, chat_common::MessageKind>::new(recv);

    send.send(Greeting(GreetingMessage {
        motd: "Hello, world!".into(),
    })).map_err({
            let log = log.clone();
            move |err| {
                error!(log, "Could not send!"; "error" => %err);
                ()
            }
        })
        .and_then(move |send| {
            recv.into_future()
                .map_err({
                    let log = log.clone();
                    move |(err, _)| {
                        error!(log, "Stream error"; "error" => %err);
                        ()
                    }
                })
                .and_then({
                    let log = log.clone();
                    move |(maybe_msg, recv)| {
                        match maybe_msg {
                            Some(Goodbye(GoodbyeMessage { ref reason })) => {
                                info!(log, "Goodbye."; "reason" => reason);
                            }

                            Some(msg) => {
                                info!(log, "Received unexpected message."; "msg" => ?msg);
                            }

                            None => {
                                error!(log, "Connection terminated unexpectedly.");
                            }
                        }
                        
                        futures::future::ok(())
                    }
                })
        })
}
