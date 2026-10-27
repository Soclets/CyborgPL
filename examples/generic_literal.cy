# A bare literal takes a generic's type, as it takes a written one (owner, 2026-10-22: "it should work"): `let $sum: <T>
# = 0;` is a d64 0 when `<T>` is filled with d64, an f64 0 with f64, an integer with i32.
function total <T: any> [lent $xs: std::list[<T>]] -> <T>:
    variable $sum: <T> = 0;
    for $x in lend $xs:
        change $sum = $sum plus $x;
    end of for
    return $sum;
end of function total
!MAIN:
    let $a: std::list[float.%decimal.64_bit] = [0.1, 0.2];
    let $b: std::list[float.binary.64_bit] = [0.5, 0.25];
    let $c: std::list[integer.signed.32_bit] = [1, 2];
    std::print.shortest(total(lend $a), " ", total(lend $b), " ", total(lend $c), newline);
!end of MAIN
