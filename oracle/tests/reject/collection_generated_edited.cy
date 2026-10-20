coll.fn c {
    for 'n' in [1, 2] {
        named "t" concat to.stored.str.utf8['n'];
        [] -> i32 { return 'n'; }
    }
    #generated_2_lines_below
    c.t1 [] -> i32 { return 1; }
    c.t2 [] -> i32 { return 20; }
}
MAIN { }
