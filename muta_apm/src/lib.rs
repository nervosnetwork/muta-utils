use rustracing_jaeger::span::{SpanReceiver, SpanSender};

const SPAN_CHANNEL_SIZE: usize = 20;

pub struct MutaTracing {
	pub span_tx: SpanSender,
}

impl MutaTracing {
	fn register() -> (Self, SpanReceiver) {
		let (span_tx, span_rx) = crossbeam_channel::bounded(SPAN_CHANNEL_SIZE);
		(MutaTracing { span_tx }, span_rx)
	}
}
