fn main() {
    let n: i64 = match std::env::var("N").ok().and_then(|s| s.parse().ok()) { Some(v) if v >= 0 && v <= 200_000_000 => v, _ => std::process::exit(2) };
    let n = n as usize;
    let mut comp = vec![false; n + 1];
    let mut count: i64 = 0;
    for p in 2..=n { if !comp[p] { count += 1; let mut m = p * p; while m <= n { comp[m] = true; m += p; } } }
    std::process::exit((count % 256) as i32);
}
