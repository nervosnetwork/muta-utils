use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

use crate::attr_parse::{parse_attrs, span_log, span_tag};

pub fn func_expand(attr: TokenStream, func: TokenStream) -> TokenStream {
    let func = parse_macro_input!(func as ItemFn);
    let func_vis = &func.vis;
    let func_block = &func.block;
    let func_decl = &func.sig;
    let func_name = &func_decl.ident;
    let (func_generics, _ty, where_clause) = &func_decl.generics.split_for_impl();
    let func_inputs = &func_decl.inputs;
    let func_output = &func_decl.output;
    let func_async = if func_decl.asyncness.is_some() {
        quote! {async}
    } else {
        quote! {}
    };

    let tracing_attrs = parse_attrs(parse_macro_input!(attr as AttributeArgs));
    let kind = tracing_attrs.kind.clone();
    let trace_name = if let Some(name) = tracing_attrs.get_tracing_name() {
        kind + "." + &name
    } else {
        kind + "." + &func_name.to_string()
    };

    let span_tag_stmts = tracing_attrs
        .get_tag_map()
        .into_iter()
        .map(|(key, val)| span_tag(key, val))
        .collect::<Vec<_>>();

    let span_log_stmts = tracing_attrs
        .get_log_map()
        .into_iter()
        .map(|(key, val)| span_log(key, val))
        .collect::<Vec<_>>();

    let res = quote! {
        #[allow(unused_variables)]
        #func_vis #func_async fn #func_name #func_generics(#func_inputs) #func_output #where_clause {
            use muta_apm::rustracing_jaeger::span::SpanContext;
            use muta_apm::rustracing::tag::Tag;
            use muta_apm::rustracing::log::LogField;

            let mut span_tags: Vec<Tag> = Vec::new();
            #(#span_tag_stmts)*

            let mut span_logs: Vec<LogField> = Vec::new();
            #(#span_log_stmts)*

            let mut span = if let Some(parent_ctx) = ctx.get::<Option<SpanContext>>("parent_span_ctx") {
                if parent_ctx.is_some() {
                    muta_apm::MUTA_TRACER.child_of_span(#trace_name, parent_ctx.clone().unwrap(), span_tags)
                } else {
                    muta_apm::MUTA_TRACER.span(#trace_name, span_tags)
                }
            } else {
                muta_apm::MUTA_TRACER.span(#trace_name, span_tags)
            };

            let ctx = match span.as_mut() {
                Some(span) => {
                    span.log(|log| {
                        for span_log in span_logs.into_iter() {
                            log.field(span_log);
                        }
                    });
                    ctx.with_value("parent_span_ctx", span.context().map(|cx| cx.clone()))
                },
                None => ctx,
            };

            #func_block
        }
    };
    res.into()
}
