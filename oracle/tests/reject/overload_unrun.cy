coll.fn ov {
    ov.flip ['x': i32] -> i32 { return 1; }
    ov.flip ['x': i64] -> i32 { return 2; }
}
MAIN {
    let 'p' := false;
    std::print.useful["before", newln];
    if 'p' { std::print.useful[ov.flip[77], newln]; }
}
