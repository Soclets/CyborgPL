# Written out for the reader. This was a `#strict` file (owner, 2026-09-27); `#strict` is gone
# (owner, 2026-10-07) and what it wrote out became the language's: `pos` on every position,
# labelled ranges, every parameter's type and a return type. `immut` and `temp` may still be
# written where they help, as here.
fn ratio [$n: i32[min: 1]] -> i32 { return 100 div.floor $n; }
MAIN {
    let immut $xs: std::list[i32] = [10, 20, 30, 40];
    let mut $total: i32 = 0;
    for temp $i: u64 [from: 1, to: 4, by: 1] {
        change $total = $total + $xs[pos: $i];
    }
    let immut $mid := std::list.slice[lend $xs, from: pos 2, to: pos 3];
    let immut $f := fn [$n: i32[min: 0, max: 1000]] -> i32 { return $n x 2; };   # the range is why 'n' times 2 fits
    std::print.useful($total, " ", $mid, " ", ratio(4), " ", $f(3), newln);
}
