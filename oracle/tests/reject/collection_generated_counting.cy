coll.fn c {
    for 'n' [from: 1, before: 3, by: 1] {
        named "t" concat to.stored.str.utf8['n'];
        [] -> i32 { return 'n'; }
    }
    #generated_3_lines_below
    c.t1 [] -> i32 { return 1; }
    c.t2 [] -> i32 { return 2; }
    c.t3 [] -> i32 { return 3; }
}
MAIN { }
