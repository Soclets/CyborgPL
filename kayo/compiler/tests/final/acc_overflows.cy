# A counted accumulator whose bound does NOT fit: up to 2^31 trips adding
# 2^62 each. The counting bound must not prove this; the final build refuses.
%extern.C fn exit [$code: i32];
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C fn atoi [$s: ptr[u8]] -> i32;
MAIN {
    let $k := "N";
    no_check let $n: i64 = to.i64(atoi(getenv(lend $k)));
    let mut $acc: i64 = 0;
    let mut $i: i64 = 1;
    while $i at_most $n {
        change $acc = $acc plus 4611686018427387904;
        change $i = $i plus 1;
    }
    no_check my.exit(to.i32($acc mod.floor 256));
}
