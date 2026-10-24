# A duration is never negative (owner, 2026-09-22): 1.5 s - 2 s is a check
# that fires. Layer 3 asked about the first side of the subtraction alone
# and "proved" it, and codegen checked only that side: it ran on negative.
extern.C fn exit [$code: i32];
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
MAIN {
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    let mut $a := 5 seconds;
    if $there { change $a = 1500 millis; }
    let $c := $a minus 2 seconds;
    if $c > 0 nanos { no_check my.exit(3); }
    no_check my.exit(4);
}
