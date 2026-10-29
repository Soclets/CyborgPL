# A type written inside a generic body names the function's blank (2026-10-12,
# corner_shop): `let 'sum': <T> = 0;` was refused as "a blank `<T>` cannot
# appear here". 1 + 2 + 3 is 6, and 40 + 2 is 42: 48.
%extern.C proc exit [let $code: i32];

set Number:
    i32;
    i64;
;set Number

fn total_of <T: Number> [lent $xs: std::ls[<T>]] -> <T>:
    var $sum: <T> = 0;
    for $x in lend $xs:
        change $sum to $sum + $x;
    ;for
    ret $sum;
;fn total_of

MAIN:
    let $a: std::ls[i32] := [1, 2, 3];
    let $b: std::ls[i64] := [40, 2];
    let $t := to.i64(total_of(lend $a)) + total_of(lend $b);
    no_check my.exit(to.i32($t));
;MAIN
