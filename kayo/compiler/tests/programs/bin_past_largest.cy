# A bin never holds an infinity (owner, 2026-09-29: a): doubling a value
# the program only learns at run time goes past the largest f64 on the
# 1024th step, and the check there stops it. Before, it became infinity
# and the loop ran on.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
proc start [] -> f64:
    let $key := "SYNAPSE_NO_SUCH_VARIABLE";
    no_check let $p := getenv(lend $key);
    if ptr.is_null($p):
        ret 1.0;
    ;if
    ret 0.5;
;proc start
MAIN:
    var $x: f64 = start();
    for $i: i32 [from: 1, to: 2000, by: 1]:
        change $x to $x x 2.0;
    ;for
    no_check my.exit(7);
;MAIN
