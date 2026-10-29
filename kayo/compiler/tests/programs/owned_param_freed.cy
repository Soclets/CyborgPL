# An owned parameter is the function's to give back when it ends: nothing
# freed a list handed over with `give`, and every such call leaked. The
# list is built at run time so the call is not worked out in advance.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
fn total [let $xs: std::ls[i32]] -> i32:
    ret $xs[pos: 1];
;fn total
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $x: i32 = 1;
    if $there:
        change $x to 2;
    ;if
    let $l: std::ls[i32] := [$x, 3];
    let $a := total(give $l);
    no_check my.exit($a - 2);
;MAIN
