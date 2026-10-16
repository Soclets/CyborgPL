collection.fn c {
    for 'n' from 1 before 3 by 1 {
        named "t" concat to.stored.str.utf8['n'];
        [] -> int32 { return 'n'; }
    }
    #generated_3_lines_below
    t1 [] -> int32 { return 1; }
    t2 [] -> int32 { return 2; }
    t3 [] -> int32 { return 3; }
}
MAIN { }
