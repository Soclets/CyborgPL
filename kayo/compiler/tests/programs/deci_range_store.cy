# A deci range type with bin's rules (owner, 2026-09-29): a value outside
# the bounds is a check site, and 2 is not in d64[min 0.0, max 1.0] (the
# oracle: "2 does not fit d64[min 0, max 1]").
%extern.C proc exit [$code: i32];
fn twice [$d: %d64] -> %d64:
    ret $d x 2;
;fn twice
MAIN:
    let $c: %d64[min: 0.0, max: 1.0] := twice(1);
    no_check my.exit(7);
;MAIN
