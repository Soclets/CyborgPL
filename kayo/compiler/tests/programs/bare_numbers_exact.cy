# Bare numbers alone are worked out exactly (owner, 2026-09-23), whatever
# is between them and wherever they stand. A sum past `i64` compared with
# 0 is just true: it stopped at run time (check claude, seed 439). A
# division by a group whose big number fits only a `u64` divides by 1:
# the build asserting its proofs read the big number as negative, divided
# by 0, and said a proof had failed (seed 371).
%extern.C proc exit [$code: i32];
MAIN:
    var $got: i32 = 0;
    if 3876992840477302775 + 7782534087963747351 > 0:
        change $got to $got + 1;
    ;if
    let $q: u64 := to.u64(0 div.floor ((2 div.floor 10047273927188643194) + 1));
    if $q == 0:
        change $got to $got + 2;
    ;if
    no_check my.exit($got);
;MAIN
