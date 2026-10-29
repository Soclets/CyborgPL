# A number or a bool as text, in its stored form (conversion.md): decimal
# digits; for a bin the full expansion of what is stored (a f32 0.1 is
# 0.100000001490116119384765625, DECISIONS 2026-09-14); true / false. Into
# a bounded text it fails out_of_range, and the text made is given back.
# The values are known only at run time: whether an unset variable is there.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
fn short [$n: i64] -> str.utf8:
    let $s := to.stored.str.utf8[min: 0, max: 3 bytes]($n)-
    otherwise:
        case out_of_range:
            ret "?";
        ;case out_of_range
    ;otherwise;
    ret give $s;
;fn short
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $got: i32 = 0;
    var $n: i64 = -9223372036854775807;
    if $there:
        change $n to $n - 1;
    ;if
    if to.stored.str.utf8($n) == "-9223372036854775808":
        change $got to $got + 1;
    ;if
    var $u: u64 = 18446744073709551614;
    if $there:
        change $u to $u + 1;
    ;if
    if to.stored.str.utf8($u) == "18446744073709551615" and to.stored.str.utf8(7) == "7":
        change $got to $got + 2;
    ;if
    var $x: f64 = 0.1;
    if $there:
        change $x to $x x 1.0;
    ;if
    if to.stored.str.utf8($x) == "0.1000000000000000055511151231257827021181583404541015625":
        change $got to $got + 4;
    ;if
    var $f: f32 = 0.1;
    if $there:
        change $f to $f x 1.0;
    ;if
    if to.stored.str.utf8($f) == "0.100000001490116119384765625":
        change $got to $got + 8;
    ;if
    var $z: f64 = -0.5;
    if $there:
        change $z to $z times.ieee 0.0;
    ;if
    if to.stored.str.utf8($z) == "-0" and to.stored.str.utf8($there) == "true" and to.stored.str.utf8(not $there) == "false":
        change $got to $got + 16;
    ;if
    var $small: i64 = 999;
    if $there:
        change $small to $small + 1;
    ;if
    if short($small) == "?" and short(-12) == "-12":
        change $got to $got + 32;
    ;if
    let $joined := "n=" concat to.stored.str.utf8($small + 4);
    if $joined == "n=1004":
        change $got to $got + 64;
    ;if
    no_check my.exit($got);
;MAIN
