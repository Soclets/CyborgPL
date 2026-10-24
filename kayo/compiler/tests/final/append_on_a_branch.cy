# A list appended to on only some trips is not in lock-step with the
# counter: its length is not n + 1, and indexing up to n must stay unproven.
extern.C fn exit [$code: i32];
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
extern.C fn atoi [$s: ptr[u8]] -> i32;
MAIN {
    let $k := "N";
    no_check let $n: i64 = to.i64(atoi(getenv(lend $k)));
    let mut $xs: std::list[bool] = [];
    let mut $i: i64 = 0;
    while $i at_most $n {
        if ($i mod.floor 2) equal_to 0 { std::list.append(lend.mut $xs, value: true); }
        change $i = $i plus 1;
    }
    let mut $p: i64 = 1;
    let mut $c: i64 = 0;
    while $p at_most $n {
        if $xs[pos: to.u64($p)] { change $c = $c plus 1; }
        change $p = $p plus 1;
    }
    no_check my.exit(to.i32($c mod.floor 256));
}
