# A loop's item has the list's element type (found by check claude, 2026-09-29): 'y' is a u8,
# so 'y' plus.wrap 10 wraps at 8 bits, and 'x' is an i32 for not.bit.
MAIN {
    let $xs: std::list[i32] = [1, 2];
    for $x in lend $xs {
        let $v: i64 = to.i64($x plus.wrap 10);
        let $w: i64 = to.i64(not.bit $x);
        std::print.useful($v, " ", $w, newln);
    }
    let $ys: std::list[u8] = [250, 3];
    for $i, $y in lend $ys { std::print.useful($i, " ", $y plus.wrap 10, newln); }
}
