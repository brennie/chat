#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use slog::Drain;
use std::net::IpAddr;
use structopt::StructOpt;

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
}
