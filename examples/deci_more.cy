# deci's leftovers (owner, 2026-09-29): mod.floor takes the divisor's sign, as integer mod.floor; pow takes a whole
# exponent and rounds like the other operations; a wider deci into a narrower one fails
# too_many_digits; a deci may be a range type, with bounds written as bins' are.
!MAIN:
    let $t: float.%deci.64 := 7.50;
    let $m: float.%deci.64 := neg 7.50;
    std::print.stored($t mod.floor 2, " ", $m mod.floor 2, " ", $t mod.floor neg 2, newln);
    let $r: float.%deci.64 := 1.05;
    std::print.stored($r pow 10, " ", $r pow 0, " ", $r pow neg 1, newln);
    let $r2: float.%deci.128 := 1.05;
    std::print.stored($r2 pow 10, newln);
    let $n: int.sign.32 := 3;
    std::print.stored($r pow $n, newln);
    let $x: float.%deci.64 := 3.141592653589793;
    let $y := to.float.%deci.32($x) otherwise 0;
    let $z: float.%deci.64 := 2.50;
    let $zz := to.float.%deci.32($z) otherwise 0;
    std::print.stored($y, " ", $zz, newln);
    let $disc: float.%deci.64[min: 0.0, max: 1.0] := 0.15;
    let $p := to.float.%deci.64[min: 0.0, max: 1.0]("1.5") otherwise $disc;
    std::print.stored($disc, " ", $p, newln);
!end of MAIN
