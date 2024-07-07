pub struct InvalidData;
pub trait EncodedSize {
   fn size(data: &[u8]) -> Result<Option<usize>, InvalidData>;
}

impl From<InvalidData> for std::io::Error {
   fn from(_value: InvalidData) -> Self {
      std::io::Error::other("invalid data.".to_string())
   }
}

macro_rules! impl_encoded_size_for_primitive {
    ($($ty:ty)*) => {
       $(
       impl EncodedSize for $ty {
         fn size(_data: &[u8]) -> Result<Option<usize>,InvalidData> {
            Ok(Some(core::mem::size_of::<$ty>()))
         }
       }
       )*
    };
}

impl_encoded_size_for_primitive! {
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

#[macro_export]
macro_rules! impl_encoded_size_for_with_len_header {
    ($($ty:ty)*) => {
       $(
       impl $crate::EncodedSize for $ty {
         fn size(mut data: &[u8]) -> Result<Option<usize>,$crate::InvalidData> {
            if data.len() < core::mem::size_of::<u32>() {
               return Ok(None);
            }
            let len = data.get_u32();
            Ok(Some(len as usize + core::mem::size_of::<u32>()))
         }
       }
       )*
    };
}
