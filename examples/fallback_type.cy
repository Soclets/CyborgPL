# A conversion's fallback takes the conversion's type (types.md: a bare literal takes its context's
# family): `to.d32($x) otherwise 0` is a d32 0, and so is 'y'. Found by check claude, 2026-09-30.
!MAIN:
    let $x: float.%deci.64 = 3.141592653589793;
    let $y := to.float.%deci.32($x) otherwise 0;
    let $z := to.int.sign.64($y times 1000000) otherwise neg 1;
    let $b: float.bin.64 = to.float.bin.64("not a number") otherwise 0;
    std::print.stored($y, " ", $z, " ", $b, newln);
!end of MAIN
