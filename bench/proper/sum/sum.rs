// sum, the proper Rust way, one thread: an iterator sum.
use std::{env, process};

fn main() {
    let n: i64 = match env::var("N").ok().and_then(|s| s.parse().ok()) {
        Some(n) if (0..=1_000_000_000).contains(&n) => n,
        _ => process::exit(2),
    };
    let acc: i64 = (1..=n).map(|i| i * i % 7).sum();
    process::exit((acc % 1_000_000_007 % 256) as i32);
}
