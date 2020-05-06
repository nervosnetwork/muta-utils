//!

use std::borrow::Cow;
use std::net::SocketAddr;

use parking_lot::RwLock;
use rustracing::sampler::AllSampler;
use rustracing_jaeger::reporter::JaegerCompactReporter;
pub use rustracing_jaeger::span::{Span, SpanContext, StartSpanOptions};
use rustracing_jaeger::Tracer;

const SPAN_CHANNEL_SIZE: usize = 20;
const _REPORTER_BUFFER_SIZE: usize = 5;

lazy_static::lazy_static! {
    pub static ref MUTA_TRACER: MutaTracer = MutaTracer::new();
}

pub struct MutaTracer {
    pub inner: RwLock<Option<Tracer>>,
}

impl MutaTracer {
    pub fn new() -> Self {
        MutaTracer {
            inner: RwLock::new(None),
        }
    }

    pub fn register(&self, service_name: String, udp_addr: SocketAddr) {
        let (span_tx, span_rx) = crossbeam_channel::bounded(SPAN_CHANNEL_SIZE);

        std::thread::spawn(move || {
            let mut reporter = JaegerCompactReporter::new(&service_name).unwrap();
            reporter
                .set_agent_addr(udp_addr)
                .expect("set upd addr error");

            loop {
                if let Ok(finished_span) = span_rx.try_recv() {
                    reporter.report(&[finished_span]).unwrap();
                }
            }
        });

        let mut tracer = self.inner.write();
        *tracer = Some(Tracer::with_sender(AllSampler, span_tx))
    }

    pub fn child_of_span<N: Into<Cow<'static, str>>>(
        &self,
        opt_name: N,
        parent_ctx: SpanContext,
    ) -> Span {
        self.inner
            .read()
            .as_ref()
            .unwrap()
            .span(opt_name)
            .child_of(&parent_ctx)
            .start()
    }

    pub fn span<N: Into<Cow<'static, str>>>(&self, opt_name: N) -> Span {
        self.inner.read().as_ref().unwrap().span(opt_name).start()
    }
}
