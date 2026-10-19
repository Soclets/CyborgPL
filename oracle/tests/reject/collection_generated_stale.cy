coll.fn c {
    for 'n' in [1, 2, 3] {
        named "t" concat to.stored.str.utf8['n'];
        [] -> int32 { return 'n'; }
    }
    #generated_2_lines_below
    c.t1 [] -> int32 { return 1; }
    c.t2 [] -> int32 { return 2; }
}
MAIN { }
