# A loop's item has the list's element type (found by check claude, 2026-09-29): 'y' is a u8,
# so 'y' plus.wrap 10 wraps at 8 bits, and 'x' is an i32 for not.bit.
!MAIN:
    let $xs: std::list[integer.signed.32_bit] = [1, 2];
    for $x in lend $xs:
        let $v: integer.signed.64_bit = to.integer.signed.64_bit($x plus.wrap 10);
        let $w: integer.signed.64_bit = to.integer.signed.64_bit(not.bit $x);
        std::print.shortest($v, " ", $w, newline);
    end of for
    let $ys: std::list[integer.unsigned.8_bit] = [250, 3];
    for $i, $y in lend $ys:
        std::print.shortest($i, " ", $y plus.wrap 10, newline);
    end of for
!end of MAIN
