// sum, the proper Rust way: scoped threads, one per core, iterator sums.
use std::{env, process, thread};

fn main() {
    let n: i64 = match env::var("N").ok().and_then(|s| s.parse().ok()) {
        Some(n) if (0..=1_000_000_000).contains(&n) => n,
        _ => process::exit(2),
    };
    let k = thread::available_parallelism().map_or(1, |x| x.get()) as i64;
    let acc: i64 = thread::scope(|s| {
        let parts: Vec<_> = (0..k)
            .map(|t| {
                let (lo, hi) = (1 + n * t / k, n * (t + 1) / k);
                s.spawn(move || (lo..=hi).map(|i| i * i % 7).sum::<i64>())
            })
            .collect();
        parts.into_iter().map(|h| h.join().unwrap()).sum()
    });
    process::exit((acc % 1_000_000_007 % 256) as i32);
}
