use crate::{
    impl_encoded_size_for_with_len_header, read_len_header_bytes, CommonDecoderState,
    CommonEncodeState, Decode, Encode,
};
use tokio_util::bytes::{Buf, BufMut, Bytes, BytesMut};

impl Encode for BytesMut {
    fn encode(
        self,
        dst: &mut BytesMut,
        _state: &mut Option<CommonEncodeState>,
    ) -> Result<(), std::io::Error> {
        dst.put_u32(self.len() as u32);
        dst.put_slice(self.chunk());
        Ok(())
    }
}

impl Decode for BytesMut {
    fn decode(
        src: &mut BytesMut,
        _state: &mut Option<CommonDecoderState>,
    ) -> Result<Option<Self>, std::io::Error>
    where
        Self: Sized,
    {
        read_len_header_bytes(src).map(|n| n)
    }
}

impl Encode for Bytes {
    fn encode(
        self,
        dst: &mut BytesMut,
        _state: &mut Option<CommonEncodeState>,
    ) -> Result<(), std::io::Error> {
        dst.put_u32(self.len() as u32);
        dst.put_slice(self.chunk());
        Ok(())
    }
}

impl Decode for Bytes {
    fn decode(
        src: &mut BytesMut,
        _state: &mut Option<CommonDecoderState>,
    ) -> Result<Option<Self>, std::io::Error>
    where
        Self: Sized,
    {
        read_len_header_bytes(src).map(|n| n.map(|n| n.freeze()))
    }
}

impl_encoded_size_for_with_len_header! {
   tokio_util::bytes::Bytes
   tokio_util::bytes::BytesMut
}
