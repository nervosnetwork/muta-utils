use proc_macro2::TokenStream;
use quote::quote;

use crate::decode::{decode_field, decode_parse_quotes, decode_unnamed_field};
use crate::encode::{encode_field, encode_unnamed_field};

pub fn impl_fixed_codec(ast: syn::DeriveInput) -> TokenStream {
	let name = ast.ident;
	let body = if let syn::Data::Struct(s) = &ast.data {
		s
	} else {
		panic!("#[derive(FixedCodec)] is only defined for structs.");
	};

	let impl_encode = impl_encode(&name, body);
	let impl_decode = impl_decode(&name, body);
	let impl_muta = quote! {
		impl muta_protocol::fixed_codec::FixedCodec for #name {
			fn encode_fixed(&self) -> ProtocolResult<bytes::Bytes> {
				Ok(bytes::Bytes::from(rlp::encode(self)))
			}

			fn decode_fixed(bytes: bytes::Bytes) -> ProtocolResult<Self> {
				Ok(rlp::decode(bytes.as_ref()).map_err(FixedCodecError::from)?)
			}
		}
	};

	quote! {
		const _: () = {
			extern crate rlp;
			extern crate muta_protocol;

			use muta_protocol::{fixed_codec::FixedCodecError, ProtocolResult, Bytes};

			#impl_encode
			#impl_decode
			#impl_muta
		};
	}
}

fn impl_encode(name: &syn::Ident, body: &syn::DataStruct) -> TokenStream {
	let is_named = match &body.fields {
		syn::Fields::Named(_) => true,
		syn::Fields::Unnamed(_) => false,
		_ => panic!("unit struct or unit variant such as None isn't supported"),
	};

	let stmts = if is_named {
		body.fields.iter().enumerate().map(|(i, field)| encode_field(i, field)).collect::<Vec<_>>()
	} else {
		body.fields
			.iter()
			.enumerate()
			.map(|(i, field)| encode_unnamed_field(i, field))
			.collect::<Vec<_>>()
	};
	let stmts_len = stmts.len();
	let stmts_len = quote! { #stmts_len };

	quote! {
		impl rlp::Encodable for #name {
			fn rlp_append(&self, stream: &mut rlp::RlpStream) {
				stream.begin_list(#stmts_len);
				#(#stmts)*
			}
		}
	}
}

pub fn impl_decode(name: &syn::Ident, body: &syn::DataStruct) -> TokenStream {
	let decoded = match &body.fields {
		syn::Fields::Named(_) => {
			let stmts = body
				.fields
				.iter()
				.enumerate()
				.map(|(i, field)| decode_field(i, field, decode_parse_quotes()))
				.collect::<Vec<_>>();

			quote! {
				#name {
				   #(#stmts)*
				}
			}
		}
		syn::Fields::Unnamed(_) => {
			let stmts = body
				.fields
				.iter()
				.enumerate()
				.map(|(i, field)| decode_unnamed_field(i, field, decode_parse_quotes()))
				.collect::<Vec<_>>();

			quote! {
				#name(#(#stmts)*)
			}
		}
		_ => panic!("unit struct or unit variant such as None isn't supported"),
	};

	quote! {
		impl rlp::Decodable for #name {
			fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
				let result = #decoded;
				Ok(result)
			}
		}
	}
}
