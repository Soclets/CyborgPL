# A whole bin or deci too big for the integer fails `out_of_range`, beside `not_whole` (owner,
# 2026-09-30: a), as text that is too big does: one `otherwise` handles both.
MAIN:
    let $b: f64 := 10000000000.0;
    let $d: %d128 := 99999999999999999999.0;
    let $small: %d64 := 42.0;
    let $x := to.i32($b) ?? -1;
    let $y := to.i64($d x 1000) ?? -2;
    let $z := to.i64($small) ?? -3;
    let $w := to.i32($d) ?? 99;
    std::print.stored($x, " ", $y, " ", $z, " ", $w, newln);
;MAIN
