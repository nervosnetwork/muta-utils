use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, AttributeArgs, Expr, ItemFn};

use crate::attr_parse::parse_attrs;

pub fn func_expand(attr: TokenStream, func: TokenStream) -> TokenStream {
    let func = parse_macro_input!(func as ItemFn);
    let func_vis = &func.vis;
    let func_block = &func.block;
    let func_decl = &func.sig;
    let func_name = &func_decl.ident;
    let func_generics = &func_decl.generics;
    let func_inputs = &func_decl.inputs;
    let func_output = &func_decl.output;
    let func_async = if func_decl.asyncness.is_some() {
        quote! {async}
    } else {
        quote! {}
    };

    let tracing_attrs = parse_attrs(parse_macro_input!(attr as AttributeArgs));
    let trace_name = if let Some(name) = tracing_attrs.get_tracing_name() {
        name
    } else {
        func_name.to_string()
    };
    let trace_child_of = if let Some(parent) = tracing_attrs.tracing_child_of.clone() {
        quote! { Some(#parent) }
    } else {
        quote! { None }
    };

    let has_tag = tracing_attrs.tracing_tag_key.is_some();
    let tag_key = if let Some(trace_tag_key) = tracing_attrs.tracing_tag_key.clone() {
        quote! { #trace_tag_key }
    } else {
        quote! { "null" }
    };

    let tag_value = if let Some(trace_tag_value) = tracing_attrs.tracing_tag_value.clone() {
        if let Ok(expr) = parse_str::<Expr>(&trace_tag_value) {
            quote! { (#expr).to_string() }
        } else {
            quote! { #trace_tag_value }
        }
    } else {
        quote! { "null" }
    };

    let res = quote! {
        #func_vis #func_async fn #func_name #func_generics(#func_inputs) #func_output {
            use crossbeam_channel::Sender;
            use rustracing::sampler::AllSampler;
            use rustracing::tag::Tag;
            use rustracing::span::FinishedSpan;
            use rustracing_jaeger::Tracer;
            use rustracing_jaeger::span::{SpanContextState, SpanContext};

            let repoter_tx = ctx.get::<Sender<FinishedSpan<SpanContextState>>>("trace_reporter_tx").unwrap().clone();
            let mut tracer = Tracer::with_sender(AllSampler, repoter_tx);
            let mut span = tracer.span(#trace_name);

            let trace_child_of: Option<&str> = #trace_child_of;
            if let Some(parent_name) = trace_child_of {
                let parent_ctx = ctx.get::<Option<&SpanContext>>(parent_name).unwrap().clone();
                if parent_ctx.is_some() {
                    span = span.child_of::<SpanContext>(parent_ctx.unwrap());
                }
            }

            if #has_tag {
                span = span.tag(Tag::new(#tag_key, #tag_value));
            }

            let span = span.start();

            // if #has_child && span.context().is_some() {
            // 	let _ = ctx.with_value(#trace_name, span.context().unwrap().clone());
            // }

            #func_block
        }
    };
    res.into()
}
