mod encoded_size;
#[cfg(feature = "xid")]
mod xid;

pub use encoded_size::*;

pub use paste::paste;
use std::io::Error;
use std::marker::PhantomData;
pub use tokio_codec_macros::{Decode, Encode};
use tokio_util::bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Default, Debug, Clone)]
pub struct CommonEncoder {}

#[derive(Debug, Clone)]
pub struct CommonDecoderState {
    pub byte_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct CommonDecoder<T> {
    pub state: Option<CommonDecoderState>,
    _marker: PhantomData<T>,
}

impl<T> From<Option<CommonDecoderState>> for CommonDecoder<T> {
    fn from(value: Option<CommonDecoderState>) -> Self {
        Self {
            state: value,
            _marker: Default::default(),
        }
    }
}

impl<T> Default for CommonDecoder<T> {
    fn default() -> Self {
        CommonDecoder {
            state: None,
            _marker: Default::default(),
        }
    }
}

macro_rules! impl_primitive_encoder {
    ($($ty:ty)*) => {
      $(
      impl Encoder<$ty> for CommonEncoder {
         type Error = std::io::Error;

         fn encode(&mut self, item: $ty, dst: &mut BytesMut) -> Result<(), Self::Error> {
            paste::paste! {
               dst.[<put_ $ty>](item);
            }
            Ok(())
         }
      }
      )*
    };
}
macro_rules! impl_primitive_decode {
    ($($ty:ty)*) => {
      $(
      impl Decode for $ty {
         fn decode(src: &mut BytesMut, _state: &mut Option<CommonDecoderState>) -> Result<Option<Self>, std::io::Error> {
            if src.len() < core::mem::size_of::<$ty>() {
               return Ok(None);
            }
            paste::paste! {
               return Ok(Some(src.[<get_ $ty>]()));
            }
         }
      }
      )*
    };
}

macro_rules! impl_primitive_codec {
    ($($ty:ty)*) => {
       impl_primitive_encoder! {
          $($ty)*
       }
       impl_primitive_decode! {
          $($ty)*
       }
    };
}

impl_primitive_codec! {
   u8
   u16
   u32
   u64
   u128
   i8
   i16
   i32
   i64
   i128
   f32
   f64
}

fn read_len_header_bytes(src: &mut BytesMut) -> Result<Option<BytesMut>, std::io::Error> {
    if src.len() < core::mem::size_of::<u32>() {
        return Ok(None);
    }
    let len = u32::from_be_bytes(src.first_chunk::<4>().unwrap().clone());
    if src.len() < (len as usize + 4) {
        return Ok(None);
    }
    let _ = src.get_u32();
    Ok(Some(src.split_to(len as usize)))
}

impl Encoder<String> for CommonEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u32(item.len() as u32);
        dst.put_slice(item.as_bytes());
        Ok(())
    }
}

impl Decode for String {
    fn decode(
        src: &mut BytesMut,
        _state: &mut Option<CommonDecoderState>,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized,
    {
        let bytes = read_len_header_bytes(src)?;
        Ok(bytes.map(|n| String::from_utf8(n.into()).unwrap()))
    }
}

impl Encoder<BytesMut> for CommonEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: BytesMut, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u32(item.len() as u32);
        dst.put_slice(item.chunk());
        Ok(())
    }
}

impl Decode for BytesMut {
    fn decode(
        src: &mut BytesMut,
        _state: &mut Option<CommonDecoderState>,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized,
    {
        read_len_header_bytes(src).map(|n| n)
    }
}

impl Encoder<Bytes> for CommonEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u32(item.len() as u32);
        dst.put_slice(item.chunk());
        Ok(())
    }
}

impl Decode for Bytes {
    fn decode(
        src: &mut BytesMut,
        _state: &mut Option<CommonDecoderState>,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized,
    {
        read_len_header_bytes(src).map(|n| n.map(|n| n.freeze()))
    }
}

pub trait Decode {
    fn decode(
        src: &mut BytesMut,
        state: &mut Option<CommonDecoderState>,
    ) -> Result<Option<Self>, std::io::Error>
    where
        Self: Sized;
}

impl<T> Decoder for CommonDecoder<T>
where
    T: Decode,
{
    type Item = T;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        T::decode(src, &mut self.state)
    }
}
