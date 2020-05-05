//!

use rustracing::sampler::AllSampler;
use rustracing_jaeger::span::SpanReceiver;
use rustracing_jaeger::Tracer;

const SPAN_CHANNEL_SIZE: usize = 20;

pub struct MutaTracer {
    pub inner: Tracer,
}

impl MutaTracer {
    pub fn register() -> (Self, SpanReceiver) {
        let (span_tx, span_rx) = crossbeam_channel::bounded(SPAN_CHANNEL_SIZE);
        let muta_tracer = MutaTracer {
            inner: Tracer::with_sender(AllSampler, span_tx),
        };
        (muta_tracer, span_rx)
    }
}
