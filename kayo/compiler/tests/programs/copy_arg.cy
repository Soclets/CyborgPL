# `copy 'l'` handed to a call is a second, independent value: the callee
# owns it and gives it back, and the caller's own list is untouched. Read as
# the same bytes, the callee freed the caller's list, which was then used
# and freed again. Built at run time so the calls are not worked out ahead.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
fn first [$xs: std::ls[i32]] -> i32:
    ret $xs[pos: 1];
;fn first
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $x: i32 = 1;
    if $there:
        change $x to 2;
    ;if
    let $l: std::ls[i32] := [$x, 3];
    let $a := first(copy $l);
    let $b := first(copy $l);
    let $c := first(give $l);
    no_check my.exit($a + $b + $c);
;MAIN
