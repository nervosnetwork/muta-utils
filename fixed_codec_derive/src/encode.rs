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
				let inner_ident = {
					if let syn::PathArguments::AngleBracketed(angle) = &top_segment.arguments {
						if let syn::GenericArgument::Type(syn::Type::Path(path)) =
							angle.args.first().expect("Vec has only one angle bracketed type; qed")
						{
							&path
								.path
								.segments
								.first()
								.expect("there must be at least 1 segment")
								.ident
						} else {
							panic!("fixed_codec_derive not supported");
						}
					} else {
						unreachable!("Vec has only one angle bracketed type; qed")
					}
				};
				quote! { stream.append_list::<#inner_ident, _>(&#id); }
			} else if ident == "Bytes" {
				let bytes = quote! { #id.to_vec() };
				quote! { stream.append(&#bytes); }
			} else {
				quote! { stream.append(&#id); }
			}
		}

		_ => panic!("fixed_codec_derive not supported"),
	}
}
