# A deci literal into a deci range is decided from its digits (check
# claude, seed 8049): 0.79 is inside [0.0, 1.0]. Precog follows no deci
# value, so a strict file refused it; now it is proven, and 3 comes back.
%extern.C proc exit [let $code: i32];
fn f [] -> i32:
    let $d: %d64[min: 0.0, max: 1.0] := 0.79;
    ret 3;
;fn f
MAIN:
    let $x: i32 := f();
    no_check my.exit($x);
;MAIN
