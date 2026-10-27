# A conversion's fallback takes the conversion's type (types.md: a bare literal takes its context's
# family): `to.d32($x) otherwise 0` is a d32 0, and so is 'y'. Found by check claude, 2026-09-30.
!MAIN:
    let $x: float.%decimal.64_bit = 3.141592653589793;
    let $y := to.float.%decimal.32_bit($x) otherwise 0;
    let $z := to.integer.signed.64_bit($y times 1000000) otherwise negative 1;
    let $b: float.binary.64_bit = to.float.binary.64_bit("not a number") otherwise 0;
    std::print.stored($y, " ", $z, " ", $b, newline);
!end of MAIN
