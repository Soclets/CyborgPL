coll.fn c {
    c ['x': int32] -> int32 { return 'x'; }
    c.q ['nope' = 1];
}
MAIN { }
