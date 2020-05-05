use std::ops::BitXor;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Lit, Meta, NestedMeta};

static TRACING_NAME: &'static str = "trace_name";
static TRACING_CHILD_OF: &'static str = "trace_child_of";
static TRACING_TAG_KEY: &'static str = "trace_tag_key";
static TRACING_TAG_VALUE: &'static str = "trace_tag_value";
static TRACING_HAS_CHILD: &'static str = "has_child";

pub struct TracingAttrs {
	pub tracing_name: Option<String>,
	pub tracing_child_of: Option<String>,
	pub tracing_tag_key: Option<String>,
	pub tracing_tag_value: Option<String>,
	pub has_child: bool,
}

impl Default for TracingAttrs {
	fn default() -> Self {
		TracingAttrs {
			tracing_name: None,
			tracing_child_of: None,
			tracing_tag_key: None,
			tracing_tag_value: None,
			has_child: false,
		}
	}
}

impl TracingAttrs {
	pub fn get_tracing_name(&self) -> Option<String> {
		self.tracing_name.clone()
	}

	pub fn get_tracing_tag(&self) -> TokenStream {
		if self.tracing_tag_key.is_some() {
			let tag_key = self.tracing_tag_key.clone().unwrap();
			let tag_value = self.tracing_tag_value.clone().unwrap();
			quote! { Some((#tag_key, #tag_value)) }
		} else {
			quote! { None }
		}
	}

	pub fn get_has_child(&self) -> TokenStream {
		let res = self.has_child;
		quote! { #res }
	}

	fn set_tracing_name(&mut self, name: String) {
		self.tracing_name = Some(name);
	}

	fn set_tracing_child_of(&mut self, child_of: String) {
		self.tracing_child_of = Some(child_of);
	}

	fn set_tracing_tag_key(&mut self, tag_key: String) {
		self.tracing_tag_key = Some(tag_key);
	}

	fn set_tracing_tag_value(&mut self, tag_value: String) {
		self.tracing_tag_value = Some(tag_value);
	}

	fn set_has_child_value(&mut self, has_child: bool) {
		self.has_child = has_child;
	}
}

pub fn parse_attrs(input: Vec<NestedMeta>) -> TracingAttrs {
	let mut attrs = TracingAttrs::default();
	for attr in input.iter() {
		match_attr(&mut attrs, attr);
	}

	if attrs.tracing_tag_key.is_some().bitxor(attrs.tracing_tag_value.is_some()) {
		panic!("Missing one of tag key or value");
	}
	attrs
}

fn match_attr(tracing_attrs: &mut TracingAttrs, input: &NestedMeta) {
	match input {
		NestedMeta::Meta(data) => match data {
			Meta::NameValue(name_value) => {
				let ident = &name_value
					.path
					.segments
					.first()
					.expect("there must be at least 1 segment")
					.ident;

				if ident == TRACING_NAME {
					tracing_attrs.set_tracing_name(get_lit_str(&name_value.lit));
				} else if ident == TRACING_CHILD_OF {
					tracing_attrs.set_tracing_child_of(get_lit_str(&name_value.lit));
				} else if ident == TRACING_TAG_KEY {
					tracing_attrs.set_tracing_tag_key(get_lit_str(&name_value.lit));
				} else if ident == TRACING_TAG_VALUE {
					tracing_attrs.set_tracing_tag_value(get_lit_str(&name_value.lit));
				} else {
					panic!("");
				}
			}
			Meta::Path(path) => {
				let ident = &path.segments.first().expect("there must be at least 1 segment").ident;
				if ident == TRACING_HAS_CHILD {
					tracing_attrs.set_has_child_value(true);
				} else {
					panic!("");
				}
			}
			_ => unreachable!(),
		},
		_ => unreachable!(),
	};
}

fn get_lit_str(lit: &Lit) -> String {
	match lit {
		Lit::Str(value) => value.value(),
		_ => unreachable!(),
	}
}
