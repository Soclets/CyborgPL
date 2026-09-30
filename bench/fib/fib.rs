fn fib(n: i64) -> i64 { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }
fn main() {
    let n: i64 = match std::env::var("N").ok().and_then(|s| s.parse().ok()) { Some(v) if v >= 0 && v <= 92 => v, _ => std::process::exit(2) };
    std::process::exit((fib(n) % 256) as i32);
}
