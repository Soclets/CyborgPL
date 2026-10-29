# A counted accumulator whose bound does NOT fit: up to 2^31 trips adding
# 2^62 each. The counting bound must not prove this; the final build refuses.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoi [let $s: ptr[u8]] -> i32;
MAIN:
    let $k := "N";
    no_check let $n: i64 := to.i64(atoi(getenv(lend $k)));
    var $acc: i64 = 0;
    var $i: i64 = 1;
    while $i <= $n:
        change $acc to $acc + 4611686018427387904;
        change $i to $i + 1;
    ;while
    no_check my.exit(to.i32($acc mod.floor 256));
;MAIN
