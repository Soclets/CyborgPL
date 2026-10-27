# A whole bin or deci too big for the integer fails `out_of_range`, beside `not_whole` (owner,
# 2026-09-30: a), as text that is too big does: one `otherwise` handles both.
!MAIN:
    let $b: float.binary.64_bit = 10000000000.0;
    let $d: float.%decimal.128_bit = 99999999999999999999.0;
    let $small: float.%decimal.64_bit = 42.0;
    let $x := to.integer.signed.32_bit($b) otherwise negative 1;
    let $y := to.integer.signed.64_bit($d times 1000) otherwise negative 2;
    let $z := to.integer.signed.64_bit($small) otherwise negative 3;
    let $w := to.integer.signed.32_bit($d) otherwise 99;
    std::print.stored($x, " ", $y, " ", $z, " ", $w, newline);
!end of MAIN
