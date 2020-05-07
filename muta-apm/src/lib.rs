//!

pub use muta_apm_derive as derive;
pub use rustracing_jaeger;

use std::borrow::Cow;
use std::net::SocketAddr;

use parking_lot::RwLock;
use rustracing::sampler::AllSampler;
use rustracing_jaeger::reporter::JaegerCompactReporter;
use rustracing_jaeger::Tracer;
use rustracing_jaeger::span::{Span, SpanContext};

const SPAN_CHANNEL_SIZE: usize = 1024 * 1024;

lazy_static::lazy_static! {
    pub static ref MUTA_TRACER: MutaTracer = MutaTracer::new();
}

pub fn global_tracer_register(service_name: &str, udp_addr: SocketAddr) {
    let (span_tx, span_rx) = crossbeam_channel::bounded(SPAN_CHANNEL_SIZE);
    let mut reporter = JaegerCompactReporter::new(service_name).unwrap();
    let mut tracer = MUTA_TRACER.inner.write();
    *tracer = Some(Tracer::with_sender(AllSampler, span_tx));

    reporter
        .set_agent_addr(udp_addr)
        .expect("set upd addr error");

    std::thread::spawn(move || loop {
        if let Ok(finished_span) = span_rx.try_recv() {
            reporter.report(&[finished_span]).unwrap();
        }
    });
}

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
    ) -> Option<Span> {
        match self.inner.read().as_ref() {
            Some(inner) => Some(inner.span(opt_name).child_of(&parent_ctx).start()),
            None => None,
        }
    }

    pub fn span<N: Into<Cow<'static, str>>>(&self, opt_name: N) -> Option<Span> {
        match self.inner.read().as_ref() {
            Some(inner) => Some(inner.span(opt_name).start()),
            None => None,
        }
    }
}
