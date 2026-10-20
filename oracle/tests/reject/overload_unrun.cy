coll.fn ov {
    ov.flip ['x': int32] -> int32 { return 1; }
    ov.flip ['x': int64] -> int32 { return 2; }
}
MAIN {
    let 'p' := false;
    std::print.useful["before", newln];
    if 'p' { std::print.useful[ov.flip[77], newln]; }
}
