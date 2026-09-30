fn main() {
    let n: i64 = match std::env::var("N").ok().and_then(|s| s.parse().ok()) { Some(v) if v >= 0 && v <= 100_000_000 => v, _ => std::process::exit(2) };
    let mut xs: Vec<i64> = Vec::new();
    for i in 1..=n { xs.push(i % 1000); }
    let mut acc: i64 = 0;
    for _ in 0..10 { for j in 0..xs.len() { acc += xs[j]; } }
    std::process::exit((acc % 256) as i32);
}
