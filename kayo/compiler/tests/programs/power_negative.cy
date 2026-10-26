# A negative exponent makes a fraction, which no integer holds: it stops.
%extern.C fn exit [$code: i32];
%extern.C fn strlen [$s: ptr[u8]] -> u64;
MAIN {
    let $s := "ab";
    no_check let $two: i64 = to.i64(strlen(lend $s));
    let $p: i64 = $two pow ($two minus 3);
    no_check my.exit(to.i32($p mod.floor 256));
}
