//!

pub use muta_apm_derive as derive;
pub use rustracing;
pub use rustracing_jaeger;

use std::borrow::Cow;
use std::net::SocketAddr;

use parking_lot::RwLock;
use rustracing::sampler::AllSampler;
use rustracing::tag::Tag;
use rustracing_jaeger::reporter::JaegerCompactReporter;
use rustracing_jaeger::span::{Span, SpanContext, TraceId};
use rustracing_jaeger::Tracer;

const SPAN_CHANNEL_SIZE: usize = 1024 * 1024;
const DEFAULT_SPAN_BATCH_SIZE: usize = 20;

lazy_static::lazy_static! {
    pub static ref MUTA_TRACER: MutaTracer = MutaTracer::new();
}

pub fn global_tracer_register(service_name: &str, udp_addr: SocketAddr, batch_size: Option<usize>) {
    let (span_tx, span_rx) = crossbeam_channel::bounded(SPAN_CHANNEL_SIZE);
    let batch_size = batch_size.unwrap_or_else(|| DEFAULT_SPAN_BATCH_SIZE);
    let mut reporter = JaegerCompactReporter::new(service_name).unwrap();
    let mut tracer = MUTA_TRACER.inner.write();
    *tracer = Some(Tracer::with_sender(AllSampler, span_tx));

    reporter
        .set_agent_addr(udp_addr)
        .expect("set udp addr error");

    let mut batch_spans = Vec::with_capacity(batch_size + 1);
    std::thread::spawn(move || loop {
        if let Ok(finished_span) = span_rx.recv() {
            batch_spans.push(finished_span);

            if batch_spans.len() >= batch_size {
                let enough_spans = batch_spans.drain(..).collect::<Vec<_>>();
                if let Err(err) = reporter.report(&enough_spans) {
                    log::warn!("jaeger report {}", err);
                }
            }
        }
    });
}

#[derive(Default)]
pub struct MutaTracer {
    pub(crate) inner: RwLock<Option<Tracer>>,
}

impl MutaTracer {
    pub fn new() -> Self {
        MutaTracer {
            inner: RwLock::new(None),
        }
    }

    pub fn child_of_span<N: Into<Cow<'static, str>>>(
        &self,
        opt_name: N,
        parent_ctx: SpanContext,
        tags: Vec<Tag>,
    ) -> Option<Span> {
        match self.inner.read().as_ref() {
            Some(inner) => {
                let mut span = inner.span(opt_name);
                for tag in tags.into_iter() {
                    span = span.tag(tag);
                }
                Some(span.child_of(&parent_ctx).start())
            }
            None => None,
        }
    }

    pub fn span<N: Into<Cow<'static, str>>>(&self, opt_name: N, tags: Vec<Tag>) -> Option<Span> {
        match self.inner.read().as_ref() {
            Some(inner) => {
                let mut span = inner.span(opt_name);
                for tag in tags.into_iter() {
                    span = span.tag(tag);
                }
                Some(span.start())
            }
            None => None,
        }
    }

    pub fn trace_id(ctx: Option<SpanContext>) -> TraceId {
        ctx.map(|c| c.state().trace_id())
            .unwrap_or_else(|| TraceId::new())
    }
}
