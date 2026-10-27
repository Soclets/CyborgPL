# A whole bin or deci too big for the integer fails `out_of_range`, beside `not_whole` (owner,
# 2026-09-30: a), as text that is too big does: one `otherwise` handles both.
!MAIN:
    let $b: float.bin.64 = 10000000000.0;
    let $d: float.%deci.128 = 99999999999999999999.0;
    let $small: float.%deci.64 = 42.0;
    let $x := to.int.sign.32($b) otherwise neg 1;
    let $y := to.int.sign.64($d times 1000) otherwise neg 2;
    let $z := to.int.sign.64($small) otherwise neg 3;
    let $w := to.int.sign.32($d) otherwise 99;
    std::print.stored($x, " ", $y, " ", $z, " ", $w, newln);
!end of MAIN
