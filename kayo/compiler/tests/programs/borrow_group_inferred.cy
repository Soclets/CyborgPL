# A borrow handed back through a call with no group (check claude,
# 2026-09-29): `b1` borrows one input, so its group is inferred (owner,
# 2026-10-07: "`b`"), and `b2`'s result is in @x.
fn b1 [lent $b: std::ls[i32]] -> lent std::ls[i32]:
    ret lend $b;
;fn b1
fn b2 [@x, @y] [lent @y $a: std::ls[i32], lent @x $b: std::ls[i32]] -> lent @x std::ls[i32]:
    ret b1(lend $b);
;fn b2
%extern.C proc exit [let $code: i32];
MAIN:
    let $s: std::ls[i32] := [1,];
    let $t: std::ls[i32] := [2, 3];
    let $r := b2(lend $s, lend $t);
    no_check my.exit(to.i32(std::ls.len(lend $r)));
;MAIN
