# A whole quotient that rounds to zero is 0, never -0 (owner, 2026-10-07: "b");
# `.ieee` keeps IEEE's -0 ("a"). 1 divided by each tells them apart: q and t
# are +0 (1), r is -0 (0): 1 + 0 + 4 = 5.
%extern.C proc exit [$code: i32];
fn h [] -> f64:
    ret 0.5;
;fn h
fn g [] -> f64:
    ret -1.75;
;fn g
fn sign [$q: f64] -> i32:
    no_check let $z: f64 := 1.0 div.near $q;
    if $z > 0.0:
        ret 1;
    ;if
    ret 0;
;fn sign
MAIN:
    let $q: f64 := h() div.ceil g();
    let $r: f64 := h() div.ceil.ieee g();
    let $t: f64 := h() div.trunc g();
    let $f: f64 := (-0.5) div.floor 1.75;
    no_check my.exit(sign($q) x 1 + sign($r) x 2 + sign($t) x 4);
;MAIN
