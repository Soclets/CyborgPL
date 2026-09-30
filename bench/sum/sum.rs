fn main() {
    let n: i64 = match std::env::var("N").ok().and_then(|s| s.parse().ok()) { Some(v) if v >= 0 && v <= 1_000_000_000 => v, _ => std::process::exit(2) };
    let mut acc: i64 = 0;
    for i in 1..=n { acc = (acc + (i * i) % 7) % 1_000_000_007; }
    std::process::exit((acc % 256) as i32);
}
