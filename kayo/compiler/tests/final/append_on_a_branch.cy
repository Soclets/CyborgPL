# A list appended to on only some trips is not in lock-step with the
# counter: its length is not n + 1, and indexing up to n must stay unproven.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C proc atoi [$s: ptr[u8]] -> i32;
MAIN:
    let $k := "N";
    no_check let $n: i64 := to.i64(atoi(getenv(lend $k)));
    var $xs: std::ls[bool] = [];
    var $i: i64 = 0;
    while $i <= $n:
        if ($i mod.floor 2) == 0:
            std::ls.append(lend.mut $xs, value: true);
        ;if
        change $i to $i + 1;
    ;while
    var $p: i64 = 1;
    var $c: i64 = 0;
    while $p <= $n:
        if $xs[pos: to.u64($p)]:
            change $c to $c + 1;
        ;if
        change $p to $p + 1;
    ;while
    no_check my.exit(to.i32($c mod.floor 256));
;MAIN
