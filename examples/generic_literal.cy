# A bare literal takes a generic's type, as it takes a written one (owner, 2026-10-22: "it should work"): `let $sum: <T>
# = 0;` is a d64 0 when `<T>` is filled with d64, an f64 0 with f64, an integer with i32.
fn total <T: any> [lent $xs: std::ls[<T>]] -> <T>:
    var $sum: <T> = 0;
    for $x in lend $xs:
        change $sum += $x;
    ;for
    ret $sum;
;fn total
MAIN:
    let $a: std::ls[%d64] := [0.1, 0.2];
    let $b: std::ls[f64] := [0.5, 0.25];
    let $c: std::ls[i32] := [1, 2];
    std::print(total(lend $a), " ", total(lend $b), " ", total(lend $c), newln);
;MAIN
