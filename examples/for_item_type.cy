# A loop's item has the list's element type (found by check claude, 2026-09-29): 'y' is a u8,
# so 'y' plus.wrap 10 wraps at 8 bits, and 'x' is an i32 for not.bit.
!MAIN:
    let $xs: std::ls[int.sign.32] = [1, 2];
    for $x in lend $xs:
        let $v: int.sign.64 = to.int.sign.64($x plus.wrap 10);
        let $w: int.sign.64 = to.int.sign.64(not.bit $x);
        std::print.shortest($v, " ", $w, newln);
    end of for
    let $ys: std::ls[int.unsign.8] = [250, 3];
    for $i, $y in lend $ys:
        std::print.shortest($i, " ", $y plus.wrap 10, newln);
    end of for
!end of MAIN
