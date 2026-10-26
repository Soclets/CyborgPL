# A whole quotient that rounds to zero is 0, never -0 (owner, 2026-10-07: "b");
# `.ieee` keeps IEEE's -0 ("a"). 1 divided by each tells them apart: q and t
# are +0 (1), r is -0 (0): 1 + 0 + 4 = 5.
%extern.C fn exit [$code: i32];
fn h [] -> f64 { return 0.5; }
fn g [] -> f64 { return -1.75; }
fn sign [$q: f64] -> i32 {
    no_check let $z: f64 = 1.0 div.near $q;
    if $z > 0.0 { return 1; }
    return 0;
}
MAIN {
    let $q: f64 = h() div.ceil g();
    let $r: f64 = h() div.ceil.ieee g();
    let $t: f64 = h() div.trunc g();
    let $f: f64 = (-0.5) div.floor 1.75;
    no_check my.exit(sign($q) times 1 plus sign($r) times 2 plus sign($t) times 4);
}
