# Recursion by induction, two findings of check claude's (seeds 3593 and a
# hand-cut one). f: a second parameter had its type looked up in MAIN, and
# Precog panicked past MAIN's one local. g: the table is filled from each
# type's floor, so g[0, 1] also ran p1 = 0, where -p1 is 0, and the
# divisor was "always zero" though no call ever passes it. f[3, 587] is 1;
# g[0, 1] is 100 mod -1, 0; g[2, 1] reaches p1 = 664 at pr = 0 (1 → 449 →
# 664): 100 mod -664 is -564 by the divisor's sign. 1 + 0 + 564 - 560 = 5.
%extern.C proc exit [let $code: i32];
fn f [let $pr: i32[min: 0, max: 6], let $p1: u64] -> i64:
    if $pr == 0:
        ret 1;
    ;if
    let $r: i64 := f($pr - 1, $p1);
    ret $r;
;fn f
fn g [let $pr: i32[min: 0, max: 6], let $p1: u64] -> i64:
    if $pr == 0:
        ret 100 mod.floor (negate to.i64($p1));
    ;if
    ret g($pr - 1, ($p1 mod.floor 233) + 448);
;fn g
MAIN:
    let $a: i64 := f(3, 587);
    let $b: i64 := g(0, 1);
    let $c: i64 := g(2, 1);
    no_check my.exit(to.i32($a + $b - $c - 560));
;MAIN
