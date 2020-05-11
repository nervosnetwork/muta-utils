use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{
    parse_macro_input, AttributeArgs, GenericArgument, ItemFn, PathArguments, ReturnType,
    TraitBound, Type, TypeParamBound, TypePath, TypeTraitObject,
};

use crate::attr_parse::{parse_attrs, span_log, span_tag};

pub fn is_ret_pin_box_fut_result(ret_ty: &ReturnType) -> (bool, bool) {
    let expect_ty = match ret_ty {
        syn::ReturnType::Type(_, ty) => ty,
        _ => return (false, false),
    };

    let expect_pin = match *(expect_ty.clone()) {
        Type::Path(TypePath { qself: _, path }) => {
            let last_seg = path.segments.last().cloned();
            match last_seg.map(|ls| (ls.ident.clone(), ls)) {
                Some((ls_ident, ls)) if ls_ident.to_string() == "Pin" => ls,
                _ => return (false, false),
            }
        }
        _ => return (false, false),
    };

    let expect_box = match &expect_pin.arguments {
        syn::PathArguments::AngleBracketed(wrapper) => match wrapper.args.last() {
            Some(GenericArgument::Type(syn::Type::Path(TypePath { qself: _, path }))) => {
                match path.segments.last().map(|ls| (ls.ident.clone(), ls)) {
                    Some((ls_ident, ls)) if ls_ident.to_string() == "Box" => ls,
                    _ => return (false, false),
                }
            }
            _ => return (false, false),
        },
        _ => return (false, false),
    };

    // Has Future trait bound
    match &expect_box.arguments {
        syn::PathArguments::AngleBracketed(wrapper) => match wrapper.args.last() {
            Some(GenericArgument::Type(syn::Type::TraitObject(TypeTraitObject {
                dyn_token: _,
                bounds,
            }))) => {
                let mut has_fut = false;
                let mut ret_result = false;

                for bound in bounds.iter() {
                    if let TypeParamBound::Trait(TraitBound { path, .. }) = bound {
                        if let Some(arg) = path.segments.last() {
                            if arg.ident.to_string() == "Future" {
                                has_fut = true;
                                ret_result = is_fut_ret_result(&arg.arguments);
                                break;
                            }
                        }
                    }
                }
                (has_fut, ret_result)
            }
            _ => (false, false),
        },
        _ => (false, false),
    }
}

pub fn func_expand(attr: TokenStream, func: TokenStream) -> TokenStream {
    let func = parse_macro_input!(func as ItemFn);
    let func_vis = &func.vis;
    let func_block = &func.block;
    let func_decl = &func.sig;
    let func_name = &func_decl.ident;
    let (func_generics, _ty, where_clause) = &func_decl.generics.split_for_impl();
    let func_inputs = &func_decl.inputs;
    let func_output = &func_decl.output;
    let is_func_ret_result = is_return_result(func_output);
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

    // Workaround for async-trait, which return Pin<Box<dyn Future>>, and cause
    // tracing span object be dropped too early.
    let (is_box_pin_fut, is_fut_ret_result) = is_ret_pin_box_fut_result(func_output);

    let func_block = if is_box_pin_fut {
        if is_fut_ret_result {
            quote! {
                Box::pin(async move {
                    let ret = #func_block.await;

                    match span.as_mut() {
                        Some(span) => {
                            match ret.as_ref() {
                                Err(e) => {
                                    span.set_tag(|| Tag::new("error", true));
                                    span.log(|log| {
                                        log.field(LogField::new(
                                            "error_msg",
                                            e.to_string(),
                                        ));
                                    });
                                    ret
                                }
                                Ok(_) => {
                                    span.set_tag(|| Tag::new("error", false));
                                    ret
                                }
                            }
                        }
                        None => ret,
                    }
                })
            }
        } else {
            quote! {
                Box::pin(async move {
                    let _ = span;
                    #func_block.await
                })
            }
        }
    } else {
        quote! {
            #func_block
        }
    };

    let func_block_report_err = if is_func_ret_result {
        quote! {
            let ret = #func_block;

            match span.as_mut() {
                Some(span) => {
                    match ret.as_ref() {
                        Err(e) => {
                            span.set_tag(|| Tag::new("error", true));
                            span.log(|log| {
                                log.field(LogField::new(
                                    "error_msg",
                                    e.to_string(),
                                ));
                            });
                            ret
                        }
                        Ok(_) => {
                            span.set_tag(|| Tag::new("error", false));
                            ret
                        }
                    }
                }
                None => ret,
            }
        }
    } else {
        quote! { #func_block }
    };

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
                    ctx.with_value("parent_span_ctx", span.context().cloned())
                },
                None => ctx,
            };

            #func_block_report_err
        }
    };
    res.into()
}

fn is_return_result(ret_type: &ReturnType) -> bool {
    match ret_type {
        ReturnType::Default => false,

        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(path) => {
                let ident = &path
                    .path
                    .segments
                    .last()
                    .expect("at least one path segment")
                    .ident;
                let re = Regex::new(r"Result$").unwrap();
                re.is_match(&ident.to_string())
            }
            _ => false,
        },
    }
}

fn is_fut_ret_result(intput: &PathArguments) -> bool {
    match intput {
        PathArguments::AngleBracketed(angle_arg) => {
            match angle_arg.args.first().expect("future output") {
                GenericArgument::Binding(binding) => match &binding.ty {
                    Type::Path(path) => path
                        .path
                        .segments
                        .last()
                        .unwrap()
                        .ident
                        .to_string()
                        .contains("Result"),
                    _ => false,
                },
                _ => false,
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use regex::Regex;

    #[test]
    fn test_regex() {
        let ret_type_1 = "ConsensusResult";
        let ret_type_2 = "ProtocolResult";

        let re = Regex::new(r"Result$").unwrap();
        assert!(re.is_match(ret_type_1));
        assert!(re.is_match(ret_type_2));

        let re = Regex::new(r"Result").unwrap();
        assert!(re.is_match("Result<(), &str>"));
    }
}
