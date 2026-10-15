// sieve, in parallel the proper Rust way: the primes up to √N first, then one Vec<bool> split into
// chunks; each scoped thread crosses off and counts its own chunk.
use std::{env, process, thread};

fn main() {
    let n: usize = match env::var("N").ok().and_then(|s| s.parse().ok()) {
        Some(n) if n <= 200_000_000 => n,
        _ => process::exit(2),
    };
    if n < 2 { process::exit(0); }
    let r = (n as f64).sqrt() as usize + 1;
    let mut small = vec![false; r + 1];
    let mut base = Vec::new();
    for p in 2..=r {
        if !small[p] {
            if p * p <= n { base.push(p); }
            for m in (p * p..=r).step_by(p) { small[m] = true; }
        }
    }
    let k = thread::available_parallelism().map_or(1, |x| x.get());
    let mut comp = vec![false; n + 1];
    let span = n - 1;
    let chunk = (span + k - 1) / k;
    let count: usize = thread::scope(|s| {
        let parts: Vec<_> = comp[2..]
            .chunks_mut(chunk.max(1))
            .enumerate()
            .map(|(t, part)| {
                let base = &base;
                s.spawn(move || {
                    let lo = 2 + t * chunk;
                    let hi = lo + part.len() - 1;
                    for &q in base {
                        let mut m = ((lo + q - 1) / q * q).max(q * q);
                        while m <= hi { part[m - lo] = true; m += q; }
                    }
                    part.iter().filter(|&&c| !c).count()
                })
            })
            .collect();
        parts.into_iter().map(|h| h.join().unwrap()).sum()
    });
    process::exit((count % 256) as i32);
}
