# `list.remove` in range, its position computed (check claude, 2026-10-08):
# 7, then 5; two left, then one: 7 + 5 x 10 + 1 x 100 = 157.
%extern.C proc exit [let $code: i32];
fn three [] -> u64:
    ret 3;
;fn three
MAIN:
    var $xs: std::ls[i32] = [5, 6, 7];
    let $k := three();
    let $a := std::ls.remove(lend.mut $xs, pos: $k);
    let $b := std::ls.remove(lend.mut $xs, pos: $k - 2);
    no_check my.exit($a + $b x 10 + to.i32(std::ls.len(lend $xs)) x 100);
;MAIN
