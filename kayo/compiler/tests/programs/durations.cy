# Durations (concurrency.md): nanoseconds in 64 bits; literals in any unit,
# added, compared across units, subtracted while it stays a length.
extern.C fn exit [$code: i32];
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
MAIN {
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    let mut $a := 5 seconds;
    if $there { change $a = 1500 millis; }
    let $b := $a plus 500 millis;
    let mut $got: i32 = 0;
    if $b equal_to 2 seconds { change $got = $got plus 1; }
    if $b > 1999999999 nanos and $b < 2001 millis { change $got = $got plus 2; }
    let $c := $b minus 1 second;
    if $c equal_to 1000 millis { change $got = $got plus 4; }
    no_check my.exit($got);
}
