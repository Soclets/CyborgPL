// fib, in parallel the proper Rust way: the top of the recursion tree split into scoped threads;
// below that, naive recursion as the program asks.
use std::{env, process, thread};

fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn par(n: u64, depth: u32) -> u64 {
    if n < 2 || depth == 0 { return fib(n); }
    thread::scope(|s| {
        let a = s.spawn(|| par(n - 1, depth - 1));
        let b = par(n - 2, depth - 1);
        a.join().unwrap() + b
    })
}

fn main() {
    let n: u64 = match env::var("N").ok().and_then(|s| s.parse().ok()) {
        Some(n) if n <= 92 => n,
        _ => process::exit(2),
    };
    process::exit((par(n, 4) % 256) as i32);
}
