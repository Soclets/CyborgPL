# A held function that can fail (functions-as-values.md: the failure list is
# part of the type): called as $f(…) with otherwise, named or a literal, a
# reason carrying its payload out.
extern.C fn exit [$code: i32];
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
fn half [$n: i32] -> i32 or fail[odd: i32] {
    if ($n mod.floor 2) equal_to 1 { fail odd[$n]; }
    return $n div.floor 2;
}
fn third [$n: i32] -> i32 or fail[odd: i32] {
    if ($n mod.floor 3) not_equal 0 { fail odd[$n]; }
    return $n div.floor 3;
}
fn apply [$f: fn[i32] -> i32 or fail[odd: i32], $n: i32] -> i32 {
    let $v := $f($n) otherwise { odd $x { return 0 minus $x; } };
    return $v;
}
MAIN {
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    let mut $n: i32 = 7;
    if $there { change $n = 12; }
    let mut $got: i32 = 0;
    if apply(half, $n) equal_to 6 and apply(half, $n plus 1) equal_to -13 { change $got = $got plus 1; }
    if apply(third, $n) equal_to 4 and apply(third, $n plus 1) equal_to -13 { change $got = $got plus 2; }
    let $g := fn [$m: i32] -> i32 or fail[odd: i32] { if $m > 10 { fail odd[$m]; } return $m; };
    if apply($g, 3) equal_to 3 and apply($g, $n) equal_to -12 { change $got = $got plus 4; }
    let $direct := $g($n) otherwise 99;
    if $direct equal_to 99 { change $got = $got plus 8; }
    no_check my.exit($got);
}
