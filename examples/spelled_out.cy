# Written out for the reader. This was a `#strict` file (owner, 2026-09-27); `#strict` is gone
# (owner, 2026-10-07) and what it wrote out became the language's: `pos` on every position,
# labelled ranges, every parameter's type and a return type. `immut` and `temp` may still be
# written where they help, as here.
fn ratio [$n: int.sign.32[min: 1]] -> int.sign.32:
    ret 100 div.floor $n;
end of fn ratio
!MAIN:
    let $xs: std::ls[int.sign.32] = [10, 20, 30, 40];
    var $total: int.sign.32 = 0;
    for temp $i: int.unsign.64 [from: 1, to: 4, by: 1]:
        change $total = $total plus $xs[pos: $i];
    end of for
    let $mid := std::ls.slice(lend $xs, from: pos 2, to: pos 3);
    let $f := fn [$n: int.sign.32[min: 0, max: 1000]] -> int.sign.32:
        ret $n times 2;
    end of fn;   # the range is why 'n' times 2 fits
    std::print.shortest($total, " ", $mid, " ", ratio(4), " ", $f(3), newln);
!end of MAIN
