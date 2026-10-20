# Every way a test ends (testing.md, owner 2026-09-29), as `cca test` reports
# it: outcomes.test-out, the oracle's words but for a check that fires, which
# CCA says in its own. A test goes on after its `must.stop` (after_stop).
fn half ['n': i32] -> i32 or fail[odd] {
    if ('n' mod.floor 2) equal_to 1 { fail odd; }
    return 'n' div.floor 2;
}
test fn plain_bool [] {
    let 'b': bool = false;
    must 'b';
}
test fn no_fail [] {
    must.fail to.i32["12"];
}
test fn no_stop [] {
    let 'xs': list[i32] = [1, 2, 3];
    must.stop 'xs'[pos 2];
}
test fn stops_anyway [] {
    let 'xs': list[i32] = [1, 2, 3];
    let 'i': i64 = 7;
    let 'y' := 'xs'[pos 'i'];
    must 'y' equal_to 1;
}
test fn fails_ok [] {
    must.fail half[3];
    must.fail.odd half[3];
    let 'h' := half[4] otherwise 0;
    must 'h' equal_to 2;
}
test fn and_expr [] {
    must 1 < 2 and 3 < 2;
}
test fn texts [] {
    let 's': str.utf8 = "abc";
    must 's' equal_to "abd";
}
test fn after_stop [] {
    let 'xs': list[i32] = [1, 2, 3];
    must.stop 'xs'[pos 99];
    must 'xs'[pos 1] equal_to 1;
    must 'xs'[pos 2] equal_to 5;
}
MAIN { }
