coll.fn f {
    f ['a': i64] -> i64 { return 1; }
    f ['b': i64] -> i64 { return 2; }
}
MAIN { std::print.useful[f[1], newln]; }
