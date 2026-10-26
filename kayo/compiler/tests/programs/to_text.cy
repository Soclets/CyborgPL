# A number or a bool as text, in its stored form (conversion.md): decimal
# digits; for a bin the full expansion of what is stored (a f32 0.1 is
# 0.100000001490116119384765625, DECISIONS 2026-09-14); true / false. Into
# a bounded text it fails out_of_range, and the text made is given back.
# The values are known only at run time: whether an unset variable is there.
%extern.C fn exit [$code: i32];
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
fn short [$n: i64] -> str.utf8 {
    let $s := to.stored.str.utf8[min: 0, max: 3 bytes]($n) otherwise { out_of_range { return "?"; } };
    return give $s;
}
MAIN {
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    let mut $got: i32 = 0;
    let mut $n: i64 = -9223372036854775807;
    if $there { change $n = $n minus 1; }
    if to.stored.str.utf8($n) equal_to "-9223372036854775808" { change $got = $got plus 1; }
    let mut $u: u64 = 18446744073709551614;
    if $there { change $u = $u plus 1; }
    if to.stored.str.utf8($u) equal_to "18446744073709551615" and to.stored.str.utf8(7) equal_to "7" { change $got = $got plus 2; }
    let mut $x: f64 = 0.1;
    if $there { change $x = $x times 1.0; }
    if to.stored.str.utf8($x) equal_to "0.1000000000000000055511151231257827021181583404541015625" { change $got = $got plus 4; }
    let mut $f: f32 = 0.1;
    if $there { change $f = $f times 1.0; }
    if to.stored.str.utf8($f) equal_to "0.100000001490116119384765625" { change $got = $got plus 8; }
    let mut $z: f64 = -0.5;
    if $there { change $z = $z times.ieee 0.0; }
    if to.stored.str.utf8($z) equal_to "-0" and to.stored.str.utf8($there) equal_to "true" and to.stored.str.utf8(not $there) equal_to "false" { change $got = $got plus 16; }
    let mut $small: i64 = 999;
    if $there { change $small = $small plus 1; }
    if short($small) equal_to "?" and short(-12) equal_to "-12" { change $got = $got plus 32; }
    let $joined := "n=" concat to.stored.str.utf8($small plus 4);
    if $joined equal_to "n=1004" { change $got = $got plus 64; }
    no_check my.exit($got);
}
