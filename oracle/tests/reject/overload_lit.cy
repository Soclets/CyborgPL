coll.fn f {
    f ['a': int64] -> int64 { return 1; }
    f ['b': int32] -> int64 { return 2; }
}
MAIN { std::print.useful[f[1], newln]; }
