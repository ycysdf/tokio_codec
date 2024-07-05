use futures_util::{SinkExt, StreamExt};
use std::net::{Ipv4Addr, SocketAddr};
use tokio_codec::{CommonDecoder, CommonEncoder};
use tokio_codec_macros::{Decode, Encode};

// Encode: impl tokio_util::codec::Encoder<Msg> for tokio_codec::CommonEncoder
// Decode: impl Decoder for CommonDecoder<Msg>
#[derive(Encode, Decode)]
enum Msg {
    A { a: u32, b: f32, d: String, e: u8 },
    B(tokio_util::bytes::Bytes),
    C(String, u32),
    D,
}

#[tokio::main]
async fn main() {
    let tcp_stream =
        tokio::net::TcpStream::connect(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0))
            .await
            .unwrap();
    let (reader, writer) = tcp_stream.into_split();
    tokio::spawn(async move {
        let mut framed_read =
            tokio_util::codec::FramedRead::new(reader, CommonDecoder::<Msg>::default());
        while let Some(msg) = framed_read.next().await {
            let _msg: Msg = msg.unwrap();
        }
    });

    let mut framed_write = tokio_util::codec::FramedWrite::new(writer, CommonEncoder::default());

    framed_write
        .send(Msg::C("XXX".to_string(), 12))
        .await
        .unwrap();
}
