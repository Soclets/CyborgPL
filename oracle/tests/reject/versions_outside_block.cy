coll.fn f {
    f ['a': i64] -> i64 { return 1; }
    f ['a': i64, 'b': i64] -> i64 { return 2; }
}
MAIN { }
