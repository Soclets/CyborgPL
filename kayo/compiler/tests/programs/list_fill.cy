# `list.fill[value, count]` (owner, 2026-09-30): a list of count copies.
# The scores are 0 but one 7 (7); three flags, all false (30, and no 50);
# no ones (0); four int8s of 100, the fourth read back (100); two bins of
# 1.5, the second doubled (3). 7 + 30 + 0 + 100 + 3 = 140.
%extern.C fn exit [$code: i32];
MAIN {
    let mut $scores := std::list.fill(value: 0, count: 5);
    change $scores[pos: 3] = 7;
    let $n: i64 = 3;
    let $flags: std::list[bool] = std::list.fill(value: false, count: $n);
    let $none: std::list[i32] = std::list.fill(value: 1, count: 0);
    let $eights: std::list[i8] = std::list.fill(value: 100, count: 4);
    let $bs: std::list[f64] = std::list.fill(value: 1.5, count: 2);
    let mut $code: i64 = 0;
    for $k: i64 [from: 1, to: 5, by: 1] {
        change $code = $code plus $scores[pos: $k];
    }
    change $code = $code plus to.i64(std::list.length(lend $flags)) times 10;
    if $flags[pos: 2] { change $code = $code plus 50; }
    change $code = $code plus to.i64(std::list.length(lend $none));
    change $code = $code plus to.i64($eights[pos: 4]);
    let $twice: i64 = to.i64($bs[pos: 2] times 2.0) otherwise 0;
    change $code = $code plus $twice;
    no_check my.exit(to.i32($code));
}
