# A loop that calls C stays in order by itself; `for para` runs it in
# slices anyway, the call's `nocheck` being the writer's word that the C
# is safe on many cores (owner, 2026-09-30). `labs` is: the sum of |i - 12345|
# mod 256 for i in 1 to 100000, as both loops give: 40 (worked out by hand;
# the oracle cannot link C).
extern.C fn exit [$code: i32];
extern.C fn labs [$n: i64] -> i64;
MAIN {
    let $n: i64 = 100000;
    let mut $a: std::list[i64] = std::list.fill(value: 0, count: $n);
    let mut $b: std::list[i64] = std::list.fill(value: 0, count: $n);
    for para $i: i64 [from: 1, to: $n, by: 1] {
        no_check let $v := labs($i minus 12345);
        change $a[pos: $i] = $v;
    }
    for $i: i64 [from: 1, to: $n, by: 1] {
        no_check let $v := labs($i minus 12345);
        change $b[pos: $i] = $v;
    }
    let mut $same: i64 = 0;
    let mut $s: i64 = 0;
    for $k: i64 [from: 1, to: $n, by: 1] {
        if $a[pos: $k] equal_to $b[pos: $k] { change $same = $same plus 1; }
        change $s = ($s plus $a[pos: $k]) mod.floor 256;
    }
    if $same not_equal $n { no_check my.exit(1); }
    no_check my.exit(to.i32($s));
}
