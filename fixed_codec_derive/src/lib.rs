//! Derive macro `#[derive(RlpFixedCodec)]`

mod decode;
mod encode;
mod fixed_codec;

extern crate proc_macro;

use proc_macro::TokenStream;

use crate::fixed_codec::impl_fixed_codec;


#[proc_macro_derive(RlpFixedCodec)]
pub fn rlp_fixed_codec(input: TokenStream) -> TokenStream {
	let input = proc_macro2::TokenStream::from(input);
	let ret = impl_fixed_codec(syn::parse2(input).unwrap());
	TokenStream::from(ret)
}
