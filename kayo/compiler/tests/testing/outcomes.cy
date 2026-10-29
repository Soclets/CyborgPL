# Every way a test ends (testing.md, owner 2026-09-29), as `cca test` reports
# it: outcomes.test-out, the oracle's words but for a check that fires, which
# CCA says in its own. A test goes on after its `must.stop` (after_stop).
fn half [let $n: i32] -> i32 or fail[odd]:
    if ($n mod.floor 2) == 1:
        fail odd;
    ;if
    ret $n div.floor 2;
;fn half
test proc plain_bool []:
    let $b: bool := false;
    must $b;
;test proc plain_bool
test proc no_fail []:
    must.fail to.i32("12");
;test proc no_fail
test proc no_stop []:
    let $xs: std::ls[i32] := [1, 2, 3];
    must.stop $xs[pos: 2];
;test proc no_stop
test proc stops_anyway []:
    let $xs: std::ls[i32] := [1, 2, 3];
    let $i: i64 := 7;
    let $y := $xs[pos: $i];
    must $y == 1;
;test proc stops_anyway
test proc fails_ok []:
    must.fail half(3);
    must.fail.odd half(3);
    let $h := half(4) ?? 0;
    must $h == 2;
;test proc fails_ok
test proc and_expr []:
    must 1 < 2 and 3 < 2;
;test proc and_expr
test proc texts []:
    let $s: str.utf8 := "abc";
    must $s == "abd";
;test proc texts
test proc after_stop []:
    let $xs: std::ls[i32] := [1, 2, 3];
    must.stop $xs[pos: 99];
    must $xs[pos: 1] == 1;
    must $xs[pos: 2] == 5;
;test proc after_stop
MAIN:
;MAIN
