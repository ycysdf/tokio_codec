use crate::{
    CommonDecoderState, CommonEncodeState, CommonEncoder, Decode, Encode, EncodedSize, InvalidData,
};
use std::io::Error;
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::Encoder;

impl Encode for xid::Id {
    fn encode(
        self,
        dst: &mut BytesMut,
        _state: &mut Option<CommonEncodeState>,
    ) -> Result<(), Error> {
        dst.put_slice(self.0.as_slice());
        Ok(())
    }
}

impl Decode for xid::Id {
    fn decode(
        src: &mut BytesMut,
        _state: &mut Option<CommonDecoderState>,
    ) -> Result<Option<Self>, std::io::Error>
    where
        Self: Sized,
    {
        if src.len() < 12 {
            Ok(None)
        } else {
            let id = src.split_to(12);
            Ok(Some(xid::Id::from_bytes(id.chunk()).unwrap()))
        }
    }
}

impl EncodedSize for xid::Id {
    fn size(data: &[u8]) -> Result<Option<usize>, InvalidData> {
        Ok(Some(12))
    }
}
