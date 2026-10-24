coll.fn f {
    versions f {
        [$a: i64] -> i64 { return 1; }
        [$b: i32] -> i64 { return 2; }
    }
}
MAIN { std::print.useful(f(1), newln); }
