use proc_macro2::TokenStream;
use quote::quote;

pub struct ParseQuotes {
	single: TokenStream,
	list: TokenStream,
}

pub fn decode_parse_quotes() -> ParseQuotes {
	ParseQuotes { single: quote! { rlp.val_at }, list: quote! { rlp.list_at } }
}

pub fn decode_unnamed_field(index: usize, field: &syn::Field, quotes: ParseQuotes) -> TokenStream {
	let index = quote! { #index };
	let single = quotes.single;
	let list = quotes.list;

	match &field.ty {
		syn::Type::Array(array) => {
			let len = &array.len;
			let bytes = quote! { rlp.val_at::<Vec<u8>>(#index)? };

			quote! { {
				let bytes: Vec<u8> = #bytes;
				if bytes.len() != #len {
					panic!("Length mismatch, got {}", bytes.len());
				}

				let mut out = [0u8; #len];
				out.copy_from_slice(&bytes);
				out
			}, }
		}
		syn::Type::Path(path) => {
			let ident =
				&path.path.segments.first().expect("there must be at least 1 segment").ident;
			let ident_type = ident.to_string();
			if ident_type == "Vec" {
				quote! { #list(#index)?, }
			} else if ident_type == "Bytes" {
				quote! { Bytes::from(rlp.val_at::<Vec<u8>>(#index)?), }
			} else if ident_type == "String" {
				let string = quote! { rlp.val_at::<String>(#index)? };
				quote! { {
					let string: String = #string;
					let ret = "0x".to_owned() + string.as_str();
					ret
				}, }
			} else {
				quote! { #single(#index)?, }
			}
		}
		_ => panic!("fixed_codec_derive not supported"),
	}
}

pub fn decode_field(index: usize, field: &syn::Field, quotes: ParseQuotes) -> TokenStream {
	let id = if let Some(ident) = &field.ident {
		quote! { #ident }
	} else {
		let index = syn::Index::from(index);
		quote! { #index }
	};

	let index = quote! { #index };
	let single = quotes.single;
	let list = quotes.list;

	match &field.ty {
		syn::Type::Array(array) => {
			let len = &array.len;
			let bytes = quote! { rlp.val_at::<Vec<u8>>(#index)? };

			quote! { #id: {
				let bytes: Vec<u8> = #bytes;
				if bytes.len() != #len {
					panic!("Length mismatch, got {}", bytes.len());
				}

				let mut out = [0u8; #len];
				out.copy_from_slice(&bytes);
				out
			}, }
		}
		syn::Type::Path(path) => {
			let ident =
				&path.path.segments.first().expect("there must be at least 1 segment").ident;
			let ident_type = ident.to_string();
			if ident_type == "Vec" {
				quote! { #id: #list(#index)?, }
			} else if ident_type == "Bytes" {
				quote! { #id: Bytes::from(rlp.val_at::<Vec<u8>>(#index)?), }
			} else {
				quote! { #id: #single(#index)?, }
			}
		}
		_ => panic!("fixed_codec_derive not supported"),
	}
}
