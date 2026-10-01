// sieve, the proper Rust way: a Vec<bool>, step_by for the multiples, filter-count at the end.
use std::{env, process};

fn main() {
    let n: usize = match env::var("N").ok().and_then(|s| s.parse().ok()) {
        Some(n) if n <= 200_000_000 => n,
        _ => process::exit(2),
    };
    let mut comp = vec![false; n + 1];
    let mut p = 2;
    while p * p <= n {
        if !comp[p] {
            for m in (p * p..=n).step_by(p) {
                comp[m] = true;
            }
        }
        p += 1;
    }
    let count = if n >= 2 { comp[2..].iter().filter(|&&c| !c).count() } else { 0 };
    process::exit((count % 256) as i32);
}
