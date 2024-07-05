use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Fields, Item};

#[proc_macro_derive(Encode)]
pub fn encode(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as Item);
    let tokens = match ast {
        Item::Enum(item) => {
            let ident = &item.ident;
            let match_items = item.variants.iter().enumerate().map(|(variant_index, variant)| {
            let variant_ident = &variant.ident;
            match &variant.fields {
               Fields::Named(named_fields) => {
                  let fields_ident: Vec<_> = named_fields.named.iter().map(|n| &n.ident).collect();
                  let fields_ty = named_fields.named.iter().map(|n| &n.ty);
                  quote! {
                     #ident::#variant_ident {
                        #(#fields_ident),*
                     } => {
                        dst.put_u8(#variant_index as u8);
                        #(
                           <tokio_codec::CommonEncoder as tokio_util::codec::Encoder<#fields_ty>>::encode(self,#fields_ident,dst)?;
                        )*
                     }
                  }
               }
               Fields::Unnamed(unnamed_fields) => {
                  let fields_ty: Vec<_> = unnamed_fields.unnamed.iter().map(|n| &n.ty).collect();
                  let fields_ident: Vec<_> = fields_ty.iter().enumerate().map(|(i, _)| format_ident!("p{}", i)).collect();
                  quote! {
                     #ident::#variant_ident(#(#fields_ident),*) => {
                        dst.put_u8(#variant_index as u8);
                        #(
                           <tokio_codec::CommonEncoder as tokio_util::codec::Encoder<#fields_ty>>::encode(self,#fields_ident,dst)?;
                        )*
                     }
                  }
               }
               Fields::Unit => quote! {
                  #ident::#variant_ident => {
                     dst.put_u8(#variant_index as u8);
                  }
               },
            }
         });
            quote! {
               impl tokio_util::codec::Encoder<#ident> for tokio_codec::CommonEncoder {
                  type Error = std::io::Error;

                  fn encode(&mut self, item: #ident, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), Self::Error> {
                     use tokio_util::bytes::BufMut;
                     match item {
                        #(#match_items),*
                     }
                     Ok(())
                  }
               }
            }
        }
        Item::Struct(item) => {
            let ident = &item.ident;
            let fields_access = item.fields.iter().enumerate().map(|(i, n)| {
                n.ident
                    .as_ref()
                    .map(|n| {
                        quote! {
                           item.#n
                        }
                    })
                    .unwrap_or_else(|| {
                        let i = syn::Index::from(i);
                        quote! {
                           item.#i
                        }
                    })
            });
            let fields_ty = item.fields.iter().map(|n| &n.ty);
            quote! {
               impl tokio_util::codec::Encoder<#ident> for tokio_codec::CommonEncoder {
                  type Error = std::io::Error;

                  fn encode(&mut self, item: #ident, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), Self::Error> {
                     use tokio_util::bytes::BufMut;
                     #(
                        <tokio_codec::CommonEncoder as tokio_util::codec::Encoder<#fields_ty>>::encode(self,#fields_access,dst)?;
                     )*
                     Ok(())
                  }
               }
            }
        }
        _n => {
            panic!("need is enum or struct")
        }
    };
    tokens.into()
}

#[proc_macro_derive(Decode)]
pub fn decode(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as Item);
    let tokens = match ast {
        Item::Enum(item) => {
            let ident = &item.ident;
            let encoded_size_match_items =
                item.variants
                    .iter()
                    .enumerate()
                    .map(|(variant_index, variant)| {
                        let stream = fields_encoded_size(variant.fields.iter().map(|n| &n.ty));
                        let variant_index = variant_index as u8;
                        quote! {
                           #variant_index => {
                              let mut sum_size = 0;
                              #stream
                              Ok(Some(sum_size + 1))
                           }
                        }
                    });
            let decode_match_items = item.variants.iter().enumerate().map(|(variant_index, variant)| {
            let variant_ident = &variant.ident;
            let variant_index = variant_index as u8;
            let fields_ty: Vec<_> = variant.fields.iter().map(|n| &n.ty).collect();

            match &variant.fields {

               Fields::Named(named_fields) => {
                  let fields_ident: Vec<_> = named_fields.named.iter().filter_map(|n| n.ident.as_ref()).collect();
                  quote! {
                     #variant_index => {
                        #(
                           let Some(#fields_ident) = <#fields_ty as tokio_codec::Decode>::decode(src,state)? else{
                              return Ok(None)
                           };
                        )*
                        Ok(Some(#ident::#variant_ident {
                           #(
                              #fields_ident
                           ),*
                        }))
                     }
                  }
               }
               Fields::Unnamed(_fields) => {
                  quote! {
                     #variant_index => {
                        Ok(Some(#ident::#variant_ident(
                           #(
                              {
                                 let Some(r) = <#fields_ty as tokio_codec::Decode>::decode(src,state)? else {
                                    return Ok(None)
                                 };
                                 r
                              }
                           ),*
                        )))
                     }
                  }
               }
               Fields::Unit => quote! {
                  #variant_index => {
                     Ok(Some(#ident::#variant_ident))
                  }
               }
            }


         });
            quote! {
                impl tokio_codec::EncodedSize for #ident {
                  fn size(mut data: &[u8]) -> Result<Option<usize>,tokio_codec::InvalidData> {
                     use tokio_util::bytes::Buf;
                     use tokio_util::bytes::BufMut;
                     if data.is_empty() {
                        return Ok(None)
                     }
                     let variant_index = data.get_u8();
                     match variant_index {
                        #(#encoded_size_match_items),*
                        _ => {
                           Err(tokio_codec::InvalidData)
                        }
                     }
                  }
               }

               impl tokio_codec::Decode for #ident {
                  fn decode(src: &mut tokio_util::bytes::BytesMut, state: &mut Option<tokio_codec::CommonDecoderState>) -> Result<Option<Self>, std::io::Error> {
                     use tokio_util::bytes::Buf;
                     if state.is_none() {
                        *state = Some(tokio_codec::CommonDecoderState {byte_count: None});
                     }
                     let tokio_codec::CommonDecoderState{byte_count} = state.as_mut().unwrap() else {
                        return Err(std::io::Error::other(format!("decode target error. should is enum.")))
                     };
                     let byte_count = if let Some(byte_count) = byte_count.clone() {
                        byte_count
                     } else {
                        if let Some(size) = <#ident as tokio_codec::EncodedSize>::size(src.chunk())? {
                           *byte_count = Some(size);
                           size
                        } else {
                           return Ok(None)
                        }
                     };
                     if src.len() < byte_count {
                        return Ok(None)
                     }
                     *state = None;
                     let variant_index = src.get_u8();
                     match variant_index {
                        #(#decode_match_items),*
                        i => {
                           Err(std::io::Error::other(format!("variant index {i:?} is invalid. ")))
                        }
                     }
                  }
               }
            }
        }
        Item::Struct(item) => {
            let ident = &item.ident;
            let fields_ty: Vec<_> = item.fields.iter().map(|n| &n.ty).collect();
            let encoded_size_impl = impl_encoded_size(ident, fields_ty.iter());
            let construct = match &item.fields {
            Fields::Named(named_fields) => {
               let fields_ident: Vec<_> = named_fields.named.iter().filter_map(|n| n.ident.as_ref()).collect();
               quote! {
                  {
                     #(
                        let Some(#fields_ident) = <#fields_ty as tokio_codec::Decode>::decode(src,state)? else{
                           return Ok(None)
                        };
                     )*
                     #ident {
                        #(
                           #fields_ident
                        ),*
                     }
                  }
               }
            }
            Fields::Unnamed(_fields) => {
               quote! {
                  #ident(
                     #(
                        {
                           let Some(r) = <#fields_ty as tokio_codec::Decode>::decode(src,state)? else {
                              return Ok(None)
                           };
                           r
                        }
                     ),*
                  )
               }
            }
            Fields::Unit => {
               return quote! {
                  #encoded_size_impl
                  impl tokio_codec::Decode for #ident {
                     fn decode(_src: &mut BytesMut, _state: &mut Option<tokio_codec::CommonDecoderState>) -> Result<Option<Self>, std::io::Error> {
                           return Ok(Some(#ident))
                     }
                  }
               }
               .into()
            }
         };
            quote! {
               #encoded_size_impl
               impl tokio_codec::Decode for #ident {
                  fn decode(src: &mut BytesMut, state: &mut Option<tokio_codec::CommonDecoderState>) -> Result<Option<Self>, std::io::Error> {
                     use tokio_util::bytes::BufMut;
                     if state.is_none() {
                        *state = Some(tokio_codec::CommonDecoderState {byte_count: None});
                     }
                     let tokio_codec::CommonDecoderState{byte_count} = state.as_mut().unwrap() else {
                        return Err(std::io::Error::other(format!("decode target error. should is struct.")))
                     };
                     let byte_count = if let Some(byte_count) = byte_count.clone() {
                        byte_count
                     } else {
                        if let Some(size) = <#ident as tokio_codec::EncodedSize>::size(src.chunk())? {
                           *byte_count = Some(size);
                           size
                        } else {
                           return Ok(None)
                        }
                     };
                     if src.len() < byte_count {
                        return Ok(None)
                     }
                     *state = None;
                     Ok(Some(#construct))
                  }
               }
            }
        }
        _n => {
            panic!("need is enum or struct")
        }
    };
    tokens.into()
}

fn impl_encoded_size<'a, T: ToTokens>(
    ident: &Ident,
    fields_ty: impl Iterator<Item = T>,
) -> proc_macro2::TokenStream {
    let stream = fields_encoded_size(fields_ty);
    quote! {
        impl tokio_codec::EncodedSize for #ident {
          fn size(mut data: &[u8]) -> Result<Option<usize>,tokio_codec::InvalidData> {
             let mut sum_size = 0;
             #stream
             Ok(Some(sum_size))
          }
       }
    }
}
fn fields_encoded_size<'a, T: ToTokens>(
    fields_ty: impl Iterator<Item = T>,
) -> proc_macro2::TokenStream {
    quote! {
       #(
          let Some(size) = <#fields_ty as tokio_codec::EncodedSize>::size(data)? else {
            return Ok(None);
          };
          sum_size += size;
          if data.len() < size {
             data = &[];
          } else {
             data = &data[size..];
          }
       )*
    }
}
