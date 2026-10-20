fn f ['a': int64] -> int64 { return 1; }
fn f ['b': bin64] -> int64 { return 2; }
MAIN { std::print.useful[f[1], newln]; }
