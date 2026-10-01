// fib: naive recursion, as the program asks.
use std::{env, process};

fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn main() {
    let n: u64 = match env::var("N").ok().and_then(|s| s.parse().ok()) {
        Some(n) if n <= 92 => n,
        _ => process::exit(2),
    };
    process::exit((fib(n) % 256) as i32);
}
