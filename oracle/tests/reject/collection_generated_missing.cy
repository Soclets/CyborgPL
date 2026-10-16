collection.fn c {
    for 'n' in [1, 2] {
        named "t" ++ to.stored.str.utf8['n'];
        [] -> int32 { return 'n'; }
    }
}
MAIN { }
