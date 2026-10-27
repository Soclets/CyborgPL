# deci's leftovers (owner, 2026-09-29): mod.floor takes the divisor's sign, as integer mod.floor; pow takes a whole
# exponent and rounds like the other operations; a wider deci into a narrower one fails
# too_many_digits; a deci may be a range type, with bounds written as bins' are.
!MAIN:
    let $t: float.%decimal.64_bit = 7.50;
    let $m: float.%decimal.64_bit = negative 7.50;
    std::print.stored($t modulo.floor 2, " ", $m modulo.floor 2, " ", $t modulo.floor negative 2, newline);
    let $r: float.%decimal.64_bit = 1.05;
    std::print.stored($r power 10, " ", $r power 0, " ", $r power negative 1, newline);
    let $r2: float.%decimal.128_bit = 1.05;
    std::print.stored($r2 power 10, newline);
    let $n: integer.signed.32_bit = 3;
    std::print.stored($r power $n, newline);
    let $x: float.%decimal.64_bit = 3.141592653589793;
    let $y := to.float.%decimal.32_bit($x) otherwise 0;
    let $z: float.%decimal.64_bit = 2.50;
    let $zz := to.float.%decimal.32_bit($z) otherwise 0;
    std::print.stored($y, " ", $zz, newline);
    let $disc: float.%decimal.64_bit[minimum: 0.0, maximum: 1.0] = 0.15;
    let $p := to.float.%decimal.64_bit[minimum: 0.0, maximum: 1.0]("1.5") otherwise $disc;
    std::print.stored($disc, " ", $p, newline);
!end of MAIN
