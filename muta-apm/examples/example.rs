use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use async_trait::async_trait;
use bytes::Bytes;
use creep::Context;

const N: u64 = 41;

struct Consensus {}

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
    muta_apm::global_tracer_register(
        "rabin_miller",
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6831),
    );
    Context::new()
}

#[muta_apm::derive::tracing_span(trace_name = "power_mod")]
pub async fn power_mod(ctx: Context, mut a: u64, mut b: u64, m: u64) -> u64 {
    let mut res = 1u64;
    std::thread::sleep(std::time::Duration::from_millis(100));
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

#[muta_apm::derive::tracing_span]
async fn rabin_miller(ctx: Context, aa: Vec<u64>, m: u64, k: u64) -> bool {
    std::thread::sleep(std::time::Duration::from_millis(200));
    for a in aa.into_iter() {
        std::thread::sleep(std::time::Duration::from_millis(100));
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

#[muta_apm::derive::tracing_span(trace_tag_key = "a", trace_tag_value = "b + 3")]
fn multi(ctx: Context, mut a: u64, mut b: u64, m: u64) -> u64 {
    std::thread::sleep(std::time::Duration::from_millis(300));
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
