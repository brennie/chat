extern crate failure;
extern crate futures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_serde_json;

use futures::stream::{SplitSink, SplitStream, Stream};
use tokio::net::TcpStream;
use tokio_io::codec::length_delimited::Framed;
use tokio_serde_json::{ReadJson, WriteJson};

pub mod messages;

pub type Send<M> = WriteJson<SplitSink<Framed<TcpStream>>, M>;
pub type Recv<M> = ReadJson<SplitStream<Framed<TcpStream>>, M>;

pub fn split_stream<RecvM, SendM>(stream: TcpStream) -> (Recv<RecvM>, Send<SendM>)
where
    for<'a> RecvM: serde::Deserialize<'a>,
    SendM: serde::Serialize,
{
    let (send, recv) = Framed::new(stream).split();

    let recv = ReadJson::<_, RecvM>::new(recv);
    let send = WriteJson::<_, SendM>::new(send);

    (recv, send)
}

pub fn join_stream<RecvM, SendM>(
    recv: Recv<RecvM>,
    send: Send<SendM>,
) -> Result<TcpStream, failure::Error> {
    recv.into_inner()
        .reunite(send.into_inner())
        .map_err(Into::into)
        .map(Framed::into_inner)
}
