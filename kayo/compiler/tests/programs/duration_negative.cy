# A duration is never negative (owner, 2026-09-22): 1.5 s - 2 s is a check
# that fires. Layer 3 asked about the first side of the subtraction alone
# and "proved" it, and codegen checked only that side: it ran on negative.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $a = 5 secs;
    if $there:
        change $a to 1500 millis;
    ;if
    let $c := $a - 2 secs;
    if $c > 0 nanos:
        no_check my.exit(3);
    ;if
    no_check my.exit(4);
;MAIN
