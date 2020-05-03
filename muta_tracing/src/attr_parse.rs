use std::ops::BitXor;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Lit, Meta, NestedMeta};

static TRACING_NAME: &'static str = "trace_name";
static TRACING_LOG: &'static str = "trace_log";
static TRACING_TAG_KEY: &'static str = "trace_tag_key";
static TRACING_TAG_VALUE: &'static str = "trace_tag_value";

pub struct TracingAttrs {
	pub tracing_name: Option<String>,
	pub tracing_log: Option<String>,
	pub tracing_tag_key: Option<String>,
	pub tracing_tag_value: Option<String>,
}

impl Default for TracingAttrs {
	fn default() -> Self {
		TracingAttrs {
			tracing_name: None,
			tracing_log: None,
			tracing_tag_key: None,
			tracing_tag_value: None,
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

	fn set_tracing_name(&mut self, name: String) {
		self.tracing_name = Some(name);
	}

	fn set_tracing_log(&mut self, log: String) {
		self.tracing_log = Some(log);
	}

	fn set_tracing_tag_key(&mut self, tag_key: String) {
		self.tracing_tag_key = Some(tag_key);
	}

	fn set_tracing_tag_value(&mut self, tag_value: String) {
		self.tracing_tag_value = Some(tag_value);
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
				} else if ident == TRACING_LOG {
					tracing_attrs.set_tracing_log(get_lit_str(&name_value.lit));
				} else if ident == TRACING_TAG_KEY {
					tracing_attrs.set_tracing_tag_key(get_lit_str(&name_value.lit));
				} else if ident == TRACING_TAG_VALUE {
					tracing_attrs.set_tracing_tag_value(get_lit_str(&name_value.lit));
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
