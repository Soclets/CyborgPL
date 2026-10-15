// lists, in parallel the proper Rust way: one Vec, split into chunks; each scoped thread fills its
// chunk and sums it ten times.
use std::{env, process, thread};

fn main() {
    let n: i64 = match env::var("N").ok().and_then(|s| s.parse().ok()) {
        Some(n) if (0..=100_000_000).contains(&n) => n,
        _ => process::exit(2),
    };
    let k = thread::available_parallelism().map_or(1, |x| x.get());
    let mut xs = vec![0i64; n as usize];
    let chunk = ((n as usize) + k - 1) / k.max(1);
    let acc: i64 = thread::scope(|s| {
        let parts: Vec<_> = xs
            .chunks_mut(chunk.max(1))
            .enumerate()
            .map(|(t, part)| {
                s.spawn(move || {
                    let base = (t * chunk) as i64;
                    for (j, x) in part.iter_mut().enumerate() {
                        *x = (base + j as i64 + 1) % 1000;
                    }
                    (0..10).map(|_| part.iter().sum::<i64>()).sum::<i64>()
                })
            })
            .collect();
        parts.into_iter().map(|h| h.join().unwrap()).sum()
    });
    process::exit((acc % 256) as i32);
}
