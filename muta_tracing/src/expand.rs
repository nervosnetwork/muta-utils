use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

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
	let trace_tag = tracing_attrs.get_tracing_tag();
	let _has_child = tracing_attrs.get_has_child();

	let res = quote! {
		#func_vis #func_async fn #func_name #func_generics(#func_inputs) #func_output {
			use crossbeam_channel::Sender;
			use rustracing::sampler::AllSampler;
			use rustracing::tag::Tag;
			use rustracing::span::FinishedSpan;
			use rustracing_jaeger::Tracer;
			use rustracing_jaeger::span::SpanContextState;

			let repoter_tx = ctx.get::<Sender<FinishedSpan<SpanContextState>>>("trace_reporter_tx").unwrap().clone();
			let mut tracer = Tracer::with_sender(AllSampler, repoter_tx);
			let mut span = tracer.span(#trace_name);

			let _trace_child_of: Option<&str> = #trace_child_of;
			// if let Some(parent_name) = trace_child_of {
			// 	let parent_ctx = ctx.get::<SpanContext<Tracing>>(parent_name).unwrap().clone();
			// 	span = span.child_of::<SpanContext<T>>(&parent_ctx.into());
			// }

			let trace_tag: Option<(&str, &str)> = #trace_tag;
			if let Some(tag) = trace_tag {
				span = span.tag(Tag::new(tag.0, tag.1));
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
