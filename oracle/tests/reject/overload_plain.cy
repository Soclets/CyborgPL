fn f [$a: i64] -> i64 { return 1; }
fn f [$b: f64] -> i64 { return 2; }
MAIN { std::print.useful(f(1), newln); }
