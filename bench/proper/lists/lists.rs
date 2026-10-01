// lists, the proper Rust way: collect an iterator into a Vec, sum with iterators.
use std::{env, process};

fn main() {
    let n: i64 = match env::var("N").ok().and_then(|s| s.parse().ok()) {
        Some(n) if (0..=100_000_000).contains(&n) => n,
        _ => process::exit(2),
    };
    let xs: Vec<i64> = (1..=n).map(|i| i % 1000).collect();
    let acc: i64 = (0..10).map(|_| xs.iter().sum::<i64>()).sum();
    process::exit((acc % 256) as i32);
}
