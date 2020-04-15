use proc_macro2::TokenStream;
use quote::quote;

pub fn encode_field(index: usize, field: &syn::Field) -> TokenStream {
	let ident = if let Some(ident) = &field.ident {
		quote! { #ident }
	} else {
		let index = syn::Index::from(index);
		quote! { #index }
	};
	let id = quote! { self.#ident };

	match &field.ty {
		syn::Type::Array(_array) => {
			let bytes = quote! { #id.to_vec() };
			quote! { stream.append(&#bytes); }
		}
		syn::Type::Path(path) => {
			let top_segment = path.path.segments.first().expect("there must be at least 1 segment");
			let ident = &top_segment.ident;
			if ident == "Vec" {
				quote! {
					let temp: Vec<Vec<u8>> = #id.iter().map(|item| item.encode_fixed().expect("fixed_codec not supported").to_vec()).collect();
					stream.append_list::<Vec<u8>, _>(&temp);
				}
			} else {
				quote! {
					let temp: Vec<u8> = #id.encode_fixed().expect("fixed_codec not supported").to_vec();
					stream.append(&temp);
				}
			}
		}

		_ => panic!("fixed_codec_derive not supported"),
	}
}
