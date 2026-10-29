# Each element written into a list is a store into its element type, a
# narrowing site like a struct field's: without it an i32 of 300 went into
# a list[i8] as 44. Known only at run time, so the check fires.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $x: i32 = 5;
    if $there:
        change $x to 300;
    ;if
    let $xs: std::ls[i8] := [1, $x];
    no_check my.exit(to.i32($xs[pos: 2]));
;MAIN
