# Written out for the reader. This was a `#strict` file (owner, 2026-09-27); `#strict` is gone
# (owner, 2026-10-07) and what it wrote out became the language's: `pos` on every position,
# labelled ranges, every parameter's type and a return type. `immut` and `temp` may still be
# written where they help, as here.
fn ratio [let $n: i32[min: 1]] -> i32:
    ret 100 div.floor $n;
;fn ratio
MAIN:
    let $xs: std::ls[i32] := [10, 20, 30, 40];
    var $total: i32 = 0;
    for temp $i: u64 [from: 1, to: 4, by: 1]:
        change $total += $xs[pos: $i];
    ;for
    let $mid := std::ls.slice(lend $xs, from: pos 2, to: pos 3);
    let $f := fn [let $n: i32[min: 0, max: 1000]] -> i32:
        ret $n x 2;
    ;fn;   # the range is why 'n' times 2 fits
    std::print($total, " ", $mid, " ", ratio(4), " ", $f(3), newln);
;MAIN
