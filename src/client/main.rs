extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;
use std::net::IpAddr;

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

    println!("{:?}", options);
}
