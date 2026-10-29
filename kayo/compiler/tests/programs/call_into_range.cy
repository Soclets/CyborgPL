# A call's answer stored into a narrower type is asked there, like any
# other value: `twice` gives 2 here (no SYNAPSE_NO_SUCH_VARIABLE is set), and
# 2 is no i32[min 0, max 1]. Filled straight into the place, it was never asked
# — the store held 2 and Precog believed the type (the oracle: "2 does not
# fit i32[min 0, max 1]").
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
fn twice [let $d: i32] -> i32:
    ret $d x 2;
;fn twice
proc one [] -> i32:
    let $key := "SYNAPSE_NO_SUCH_VARIABLE";
    no_check let $p := getenv(lend $key);
    if ptr.is_null($p):
        ret 1;
    ;if
    ret 0;
;proc one
MAIN:
    let $c: i32[min: 0, max: 1] := twice(one());
    no_check my.exit(7);
;MAIN
