use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_str, Expr, Lit, Meta, NestedMeta};

static KIND: &str = "kind";
static TRACING_NAME: &str = "trace_name";
static TRACING_TAG_KEYS: &str = "trace_tag_keys";
static TRACING_TAG_VALUES: &str = "trace_tag_values";
static TRACING_LOG_KEYS: &str = "trace_log_keys";
static TRACING_LOG_VALUES: &str = "trace_log_values";

pub struct TracingAttrs {
    pub kind:               String,
    pub tracing_name:       Option<String>,
    pub tracing_tag_keys:   Vec<String>,
    pub tracing_tag_values: Vec<String>,
    pub tracing_log_keys:   Option<Vec<String>>,
    pub tracing_log_values: Option<Vec<String>>,
}

impl Default for TracingAttrs {
    fn default() -> Self {
        TracingAttrs {
            kind:               String::new(),
            tracing_name:       None,
            tracing_tag_keys:   Vec::new(),
            tracing_tag_values: Vec::new(),
            tracing_log_keys:   None,
            tracing_log_values: None,
        }
    }
}

impl TracingAttrs {
    pub fn get_tracing_name(&self) -> Option<String> {
        self.tracing_name.clone()
    }

    pub fn get_tag_map(&self) -> HashMap<String, String> {
        let mut keys = self.tracing_tag_keys.clone();
        let mut values = self.tracing_tag_values.clone();

        keys.push("kind = ".to_string());
        values.push(self.kind.clone());

        assert!(keys.len() == values.len());

        keys.iter()
            .zip(values.iter())
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<HashMap<_, _>>()
    }

    pub fn get_log_map(&self) -> HashMap<String, String> {
        if self.tracing_log_keys.is_none() && self.tracing_log_values.is_none() {
            return HashMap::new();
        }

        let keys = self.tracing_log_keys.clone().unwrap();
        let values = self.tracing_log_values.clone().unwrap();
        assert!(keys.len() == values.len());

        keys.iter()
            .zip(values.iter())
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<HashMap<_, _>>()
    }

    fn set_kind(&mut self, kind: String) {
        self.kind = kind;
    }

    fn set_tracing_name(&mut self, name: String) {
        self.tracing_name = Some(name);
    }

    fn set_tracing_tag_keys(&mut self, tag_keys: Vec<String>) {
        self.tracing_tag_keys = tag_keys;
    }

    fn set_tracing_tag_values(&mut self, tag_values: Vec<String>) {
        self.tracing_tag_values = tag_values;
    }

    fn set_tracing_log_keys(&mut self, log_keys: Vec<String>) {
        self.tracing_log_keys = Some(log_keys);
    }

    fn set_tracing_log_values(&mut self, log_values: Vec<String>) {
        self.tracing_log_values = Some(log_values);
    }
}

pub fn parse_attrs(input: Vec<NestedMeta>) -> TracingAttrs {
    let mut attrs = TracingAttrs::default();
    for attr in input.iter() {
        match_attr(&mut attrs, attr);
    }

    attrs
}

pub fn span_log(key: String, val: String) -> TokenStream {
    if let Ok(expr) = parse_str::<Expr>(&val) {
        quote! { span_logs.push(LogField::new(#key, (#expr).to_string())); }
    } else {
        quote! { span_logs.push(LogField::new(#key, #val)); }
    }
}

pub fn span_tag(key: String, val: String) -> TokenStream {
    quote! { span_tags.push(Tag::new(#key, #val)); }
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

                if ident == KIND {
                    tracing_attrs.set_kind(get_lit_str(&name_value.lit));
                } else if ident == TRACING_NAME {
                    tracing_attrs.set_tracing_name(get_lit_str(&name_value.lit));
                } else if ident == TRACING_TAG_KEYS {
                    tracing_attrs.set_tracing_tag_keys(parse_list(&get_lit_str(&name_value.lit)));
                } else if ident == TRACING_TAG_VALUES {
                    tracing_attrs.set_tracing_tag_values(parse_list(&get_lit_str(&name_value.lit)));
                } else if ident == TRACING_LOG_KEYS {
                    tracing_attrs.set_tracing_log_keys(parse_list(&get_lit_str(&name_value.lit)));
                } else if ident == TRACING_LOG_VALUES {
                    tracing_attrs.set_tracing_log_values(parse_list(&get_lit_str(&name_value.lit)));
                } else {
                    panic!("");
                }
            }
            _ => unreachable!("name_value"),
        },
        _ => unreachable!("meta"),
    };
}

fn get_lit_str(lit: &Lit) -> String {
    match lit {
        Lit::Str(value) => value.value(),
        _ => unreachable!("lit_str"),
    }
}

fn parse_list(input: &str) -> Vec<String> {
    match parse_str::<Expr>(&input).expect("parse list error") {
        Expr::Array(expr_elems) => expr_elems
            .elems
            .iter()
            .map(|elem| elem.to_token_stream().to_string())
            .collect::<Vec<_>>(),
        _ => unreachable!("array"),
    }
}
