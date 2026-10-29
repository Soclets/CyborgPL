# A group of bare numbers beside a wrapping operator's typed side is plain
# maths, as wide as its own values need, and wraps to the operator's width
# (check claude, seed 34247): 5 + 2^63, 5 + (2^64 + 1) = 6, 5 + (-1) = 4,
# and (-1) + 5 = 4: 13 + 60 + 400 + 4000 = 4473, exit 121 (the oracle's).
%extern.C proc exit [let $code: i32];
fn five [] -> u64:
    ret 5;
;fn five
MAIN:
    let $v: u64 := five();
    let $a := $v plus.wrap (9223372036854775807 + 1);
    let $b := $v plus.wrap (18446744073709551615 + 2);
    let $c := $v plus.wrap (0 - 1);
    let $d := (0 - 1) plus.wrap $v;
    no_check my.exit(to.i32(($a mod.floor 100) + $b x 10 + $c x 100 + $d x 1000));
;MAIN
