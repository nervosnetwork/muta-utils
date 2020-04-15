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
				let field_ident = match &path
					.path
					.segments
					.first()
					.expect("there must be at least 1 segment")
					.arguments
				{
					syn::PathArguments::AngleBracketed(argc) => {
						argc.args.first().expect("there must be at least 1 argument")
					}
					_ => panic!("there must be at least 1 type"),
				};

				quote! { {
					let temp: Vec<Vec<u8>> = #list(#index)?;
					temp
						.iter()
						.map(|item| #field_ident::decode_fixed(Bytes::from(item.clone())).expect("decode error"))
						.collect::<Vec<_>>()
				}, }
			} else {
				quote! { {
					let bytes: Vec<u8> = #single(#index)?;
					#ident::decode_fixed(Bytes::from(bytes)).expect("decode error")
				}, }
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
				let field_ident = match &path
					.path
					.segments
					.first()
					.expect("there must be at least 1 segment")
					.arguments
				{
					syn::PathArguments::AngleBracketed(argc) => {
						argc.args.first().expect("there must be at least 1 argument")
					}
					_ => panic!("there must be at least 1 type"),
				};

				quote! { #id: {
					let temp: Vec<Vec<u8>> = #list(#index)?;
					temp
						.into_iter()
						.map(|bytes| #field_ident::decode_fixed(Bytes::from(bytes)).expect("decode error"))
						.collect::<Vec<_>>()
				}, }
			} else {
				quote! { #id: {
					let bytes: Vec<u8> = #single(#index)?;
					#ident::decode_fixed(Bytes::from(bytes)).expect("decode error")
				}, }
			}
		}
		_ => panic!("fixed_codec_derive not supported"),
	}
}
