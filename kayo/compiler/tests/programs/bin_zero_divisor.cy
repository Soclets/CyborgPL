# A bin divided by zero is no number (owner, 2026-09-29: a): 0.0 div 0.0
# would be a NaN, and the divisor is asked first. The divisor comes from
# run time, so the check is the program's.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
proc divisor [] -> f64:
    let $key := "SYNAPSE_NO_SUCH_VARIABLE";
    no_check let $p := getenv(lend $key);
    if ptr.is_null($p):
        ret 0.0;
    ;if
    ret 2.0;
;proc divisor
MAIN:
    let $d := divisor();
    let $q: f64 := 1.0 div.near $d;
    no_check my.exit(7);
;MAIN
