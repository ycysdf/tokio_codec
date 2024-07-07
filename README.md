# tokio_x_codec

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ycysdf/tokio_x_codec#LICENSE)

[//]: # ([![Crates.io]&#40;https://img.shields.io/crates/v/tokio_x_codec.svg&#41;]&#40;https://crates.io/crates/tokio_x_codec&#41;)
[//]: # ([![Docs]&#40;https://docs.rs/tokio_x_codec/badge.svg&#41;]&#40;https://docs.rs/tokio_x_codec&#41;)

## Example

```rust
use futures_util::{SinkExt, StreamExt};
use std::io::Error;
use std::net::{Ipv4Addr, SocketAddr};
use tokio_x_codec::{
   CommonDecoder, CommonDecoderState, CommonEncodeState, CommonEncoder, Decode, Encode,
   EncodedSize, InvalidData,
};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

struct Foo(u32);

impl Encode for Foo {
   fn encode(
      self,
      dst: &mut BytesMut,
      _state: &mut Option<CommonEncodeState>,
   ) -> Result<(), Error> {
      dst.put_u32(self.0);
      Ok(())
   }
}
impl Decode for Foo {
   fn decode(
      src: &mut BytesMut,
      _state: &mut Option<CommonDecoderState>,
   ) -> Result<Option<Self>, Error>
   where
      Self: Sized,
   {
      Ok(Some(Foo(src.get_u32())))
   }
}
impl EncodedSize for Foo {
   fn size(data: &[u8]) -> Result<Option<usize>, InvalidData> {
      Ok(Some(core::mem::size_of::<u32>()))
   }
}

// Encode: impl tokio_util::codec::Encoder<Msg> for tokio_x_codec::CommonEncoder
// Decode: impl Decoder for CommonDecoder<Msg>
#[derive(Encode, Decode)]
enum Msg {
   A { a: u32, b: f32, d: String, e: u8 },
   B(tokio_util::bytes::Bytes),
   C(String, u32),
   D,
   E(Foo),
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
```