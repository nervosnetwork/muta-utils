use std::boxed::Box;
use std::sync::Arc;

use creep::Context;
use muta_tracing::tracing_span;
use skywalking_core::skywalking::agent::reporter::Reporter;
use skywalking_core::skywalking::core::{ContextListener, TracingContext};

const N: u64 = 41;

#[derive(Clone)]
pub struct SkyWalkingReporter {
	inner: Arc<Reporter>,
}

impl std::fmt::Debug for SkyWalkingReporter {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		f.write_fmt(format_args!(
			"SkyWalkingReporter ID {:?}",
			self.inner.as_ref().service_instance_id()
		))
	}
}

impl SkyWalkingReporter {
	pub fn new() -> Self {
		SkyWalkingReporter { inner: Arc::new(Reporter::new()) }
	}

	pub fn report_trace(&self, finish_ctx: Box<TracingContext>) {
		self.inner.report_trace(finish_ctx);
	}
}

#[tokio::main]
async fn main() {
	let ctx = init_ctx();

	let mut k = 0;
	let mut m = N - 1;
	let aa = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

	while (m & 1) == 0 {
		m >>= 1;
		k += 1;
	}

	let res = rabin_miller(ctx, aa, m, k).await;

	println!("{} is prime number {:?}", N, res);
}

fn init_ctx() -> Context {
	Context::new()
		.with_value("trace_id", 0i32)
		.with_value("trace_reporter", SkyWalkingReporter::new())
}

#[tracing_span(trace_name = "power_mod")]
pub async fn power_mod(ctx: Context, mut a: u64, mut b: u64, m: u64) -> u64 {
	let mut res = 1u64;
	a %= m;
	while b != 0 {
		if (b & 1) == 1 {
			res = multi(ctx.clone(), res, a, m);
			b -= 1;
		}
		b >>= 1;
		a = multi(ctx.clone(), a, a, m);
	}
	res
}

#[tracing_span]
async fn rabin_miller(ctx: Context, aa: Vec<u64>, m: u64, k: u64) -> bool {
	for a in aa.into_iter() {
		let mut x = power_mod(ctx.clone(), a, m, N).await;
		let mut y: u64 = 0;
		for _i in 0..k {
			y = multi(ctx.clone(), x, x, N);
			if y == 1 && x != 1 && x != (N - 1) {
				return false;
			}
			x = y;
		}
		if y != 1 {
			return false;
		}
	}
	true
}

#[tracing_span(trace_tag_key = "a", trace_tag_value = "b")]
fn multi(ctx: Context, mut a: u64, mut b: u64, m: u64) -> u64 {
	let mut res = 0u64;
	a %= m;
	while b != 0 {
		if (b & 1) == 1 {
			res = (res + a) % m;
			b -= 1;
		}
		b >>= 1;
		a = (a + a) % m;
	}
	res
}
