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
	let _trace_log = if let Some(log) = tracing_attrs.tracing_log.clone() {
		quote! { Some(#log)}
	} else {
		quote! { None }
	};
	let trace_tag = tracing_attrs.get_tracing_tag();

	let res = quote! {
		#func_vis #func_async fn #func_name #func_generics(#func_inputs) #func_output {
			use skywalking_core::skywalking::core as skywalkingcore;
			use skywalking_core::skywalking::agent as skywalkingagent;
			use skywalking_core::skywalking::core::{Context, ContextListener};

			let service_instance_id = *ctx.get::<i32>("trace_id").unwrap();
			let repoter = ctx.get::<SkyWalkingReporter>("trace_reporter").unwrap().clone();
			let mut tracing_context = skywalkingcore::TracingContext::new(Some(service_instance_id)).unwrap();
			let mut span = tracing_context.create_entry_span(#trace_name, None, None);

			let trace_tag: Option<(&str, &str)> = #trace_tag;
			if let Some(tag) = trace_tag {
				span.tag(skywalkingcore::Tag::new(String::from(tag.0), String::from(tag.1)));
			}

			let func_ret = #func_block;

			tracing_context.finish_span(span);
			repoter.report_trace(std::boxed::Box::new(tracing_context));

			func_ret
		}
	};
	res.into()
}
