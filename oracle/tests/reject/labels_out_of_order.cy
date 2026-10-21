coll.fn say {
    labels ['a' as first, 'b' as second];
    say ['a': i32, 'b': i32] -> i32 { return 'a'; }
}
MAIN { let 'x' := say[second: 2, first: 1]; }
