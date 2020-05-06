//!

use std::borrow::Cow;

use rustracing::sampler::AllSampler;
use rustracing_jaeger::reporter::JaegerCompactReporter;
pub use rustracing_jaeger::span::StartSpanOptions;
use rustracing_jaeger::Tracer;

const SPAN_CHANNEL_SIZE: usize = 20;
const REPORTER_BUFFER_SIZE: usize = 5;

pub struct MutaTracer {
    pub inner: Tracer,
}

impl MutaTracer {
    pub fn new() -> Self {
        let (span_tx, span_rx) = crossbeam_channel::bounded(SPAN_CHANNEL_SIZE);

        std::thread::spawn(move || {
            let reporter = JaegerCompactReporter::new("muta-chian").unwrap();
            let mut reporter_buffer = Vec::with_capacity(REPORTER_BUFFER_SIZE);

            loop {
                if let Ok(finished_span) = span_rx.try_recv() {
                    reporter_buffer.push(finished_span);
                    if reporter_buffer.len() == REPORTER_BUFFER_SIZE {
                        reporter.report(&reporter_buffer).unwrap();
                        reporter_buffer.clear();
                    }
                }
            }
        });

        MutaTracer {
            inner: Tracer::with_sender(AllSampler, span_tx),
        }
    }

    pub fn span<N: Into<Cow<'static, str>>>(&self, opt_name: N) -> StartSpanOptions {
        self.inner.span(opt_name)
    }
}
