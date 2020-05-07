use std::collections::HashMap;
use std::ops::BitXor;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, Expr, Lit, Meta, NestedMeta};

static TRACING_NAME: &str = "trace_name";
static TRACING_TAG_KEY: &str = "trace_tag_key";
static TRACING_TAG_VALUE: &str = "trace_tag_value";

pub struct TracingAttrs {
    pub tracing_name:      Option<String>,
    pub tracing_tag_key:   Option<Vec<String>>,
    pub tracing_tag_value: Option<Vec<String>>,
}

impl Default for TracingAttrs {
    fn default() -> Self {
        TracingAttrs {
            tracing_name:      None,
            tracing_tag_key:   None,
            tracing_tag_value: None,
        }
    }
}

impl TracingAttrs {
    pub fn get_tracing_name(&self) -> Option<String> {
        self.tracing_name.clone()
    }

    pub fn get_tag_map(&self) -> HashMap<String, String> {
        if self.tracing_tag_key.is_none() && self.tracing_tag_value.is_none() {
            return HashMap::new();
        }

        let keys = self.tracing_tag_key.clone().unwrap();
        let values = self.tracing_tag_value.clone().unwrap();
        assert!(keys.len() == values.len());

        keys.iter()
            .zip(values.iter())
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<HashMap<_, _>>()
    }

    fn set_tracing_name(&mut self, name: String) {
        self.tracing_name = Some(name);
    }

    fn set_tracing_tag_key(&mut self, tag_key: Vec<String>) {
        self.tracing_tag_key = Some(tag_key);
    }

    fn set_tracing_tag_value(&mut self, tag_value: Vec<String>) {
        self.tracing_tag_value = Some(tag_value);
    }
}

pub fn parse_attrs(input: Vec<NestedMeta>) -> TracingAttrs {
    let mut attrs = TracingAttrs::default();
    for attr in input.iter() {
        match_attr(&mut attrs, attr);
    }

    if attrs
        .tracing_tag_key
        .is_some()
        .bitxor(attrs.tracing_tag_value.is_some())
    {
        panic!("Missing one of tag key or value");
    }
    attrs
}

pub fn span_tag(key: &str, val: &str) -> TokenStream {
    let expr = parse_str::<Expr>(&val).unwrap();
    quote! { span.tag(rustracing::tag::Tag::new(&#key, (#expr).to_string())); }
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
                } else if ident == TRACING_TAG_KEY {
                    tracing_attrs.set_tracing_tag_key(parse_list(&get_lit_str(&name_value.lit)));
                } else if ident == TRACING_TAG_VALUE {
                    tracing_attrs.set_tracing_tag_value(parse_list(&get_lit_str(&name_value.lit)));
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

fn parse_list(input: &str) -> Vec<String> {
    match parse_str::<Expr>(&input).unwrap() {
        Expr::Array(expr_elems) => expr_elems
            .elems
            .iter()
            .map(|elem| match elem {
                Expr::Lit(elem_lit) => get_lit_str(&elem_lit.lit),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>(),
        _ => unreachable!(),
    }
}
