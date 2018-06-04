extern crate structopt;
#[macro_use]
extern crate structopt_derive;

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

fn main() {
    let options = Options::from_args();

    println!("{:?}", options);
}
