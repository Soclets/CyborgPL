coll.fn c {
    for $n in [1, 2] {
        named "t" concat to.stored.str.utf8($n);
        [] -> i32 { return $n; }
    }
}
MAIN { }
