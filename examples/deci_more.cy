# deci's leftovers (owner, 2026-09-29): mod.floor takes the divisor's sign, as integer mod.floor; pow takes a whole
# exponent and rounds like the other operations; a wider deci into a narrower one fails
# too_many_digits; a deci may be a range type, with bounds written as bins' are.
MAIN:
    let $t: %d64 := 7.50;
    let $m: %d64 := -7.50;
    std::println.stored($t mod.floor 2, " ", $m mod.floor 2, " ", $t mod.floor -2);
    let $r: %d64 := 1.05;
    std::println.stored($r ^ 10, " ", $r ^ 0, " ", $r ^ -1);
    let $r2: %d128 := 1.05;
    std::println.stored($r2 ^ 10);
    let $n: i32 := 3;
    std::println.stored($r ^ $n);
    let $x: %d64 := 3.141592653589793;
    let $y := to.%d32($x) ?? 0;
    let $z: %d64 := 2.50;
    let $zz := to.%d32($z) ?? 0;
    std::println.stored($y, " ", $zz);
    let $disc: %d64[min: 0.0, max: 1.0] := 0.15;
    let $p := to.%d64[min: 0.0, max: 1.0]("1.5") ?? $disc;
    std::println.stored($disc, " ", $p);
;MAIN
