# A `lent.mut` borrow handed back changes what was lent (owner, 2026-09-29):
# 'xs' becomes [7, 2, 9], so 7 x 10 + 3 = 73 (the oracle agrees).
%extern.C proc exit [$code: i32];
proc pickmut [@x] [lent.mut @x $a: std::ls[i32], lent $b: std::ls[i32]] -> lent.mut @x std::ls[i32]:
    ret lend.mut $a;
;proc pickmut
proc run [] -> i32:
    var $xs: std::ls[i32] = [1, 2];
    let $ys: std::ls[i32] := [5, 6];
    var $m = pickmut(lend.mut $xs, lend $ys);
    change $m[pos: 1] to 7;
    std::ls.append(lend.mut $m, value: 9);
    ret $xs[pos: 1] x 10 + to.i32(std::ls.len(lend $xs));
;proc run
MAIN:
    no_check my.exit(run());
;MAIN
