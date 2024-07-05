use crate::{CommonDecoderState, CommonEncoder, Decode, EncodedSize, InvalidData};
use tokio_util::bytes::{BufMut, BytesMut};
use tokio_util::codec::Encoder;

impl Encoder<xid::Id> for CommonEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: xid::Id, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_slice(item.0.as_slice());
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
