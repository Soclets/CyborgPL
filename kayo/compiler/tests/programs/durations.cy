# Durations (concurrency.md): nanoseconds in 64 bits; literals in any unit,
# added, compared across units, subtracted while it stays a length.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $a = 5 secs;
    if $there:
        change $a to 1500 millis;
    ;if
    let $b := $a + 500 millis;
    var $got: i32 = 0;
    if $b == 2 secs:
        change $got to $got + 1;
    ;if
    if $b > 1999999999 nanos and $b < 2001 millis:
        change $got to $got + 2;
    ;if
    let $c := $b - 1 sec;
    if $c == 1000 millis:
        change $got to $got + 4;
    ;if
    no_check my.exit($got);
;MAIN
