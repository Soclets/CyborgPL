# `list.fill[value, count]` (owner, 2026-09-30): a list of count copies.
# The scores are 0 but one 7 (7); three flags, all false (30, and no 50);
# no ones (0); four int8s of 100, the fourth read back (100); two bins of
# 1.5, the second doubled (3). 7 + 30 + 0 + 100 + 3 = 140.
%extern.C proc exit [let $code: i32];
MAIN:
    var $scores = std::ls.fill(value: 0, count: 5);
    change $scores[pos: 3] to 7;
    let $n: i64 := 3;
    let $flags: std::ls[bool] := std::ls.fill(value: false, count: $n);
    let $none: std::ls[i32] := std::ls.fill(value: 1, count: 0);
    let $eights: std::ls[i8] := std::ls.fill(value: 100, count: 4);
    let $bs: std::ls[f64] := std::ls.fill(value: 1.5, count: 2);
    var $code: i64 = 0;
    for $k: i64 [from: 1, to: 5, by: 1]:
        change $code to $code + $scores[pos: $k];
    ;for
    change $code to $code + to.i64(std::ls.len(lend $flags)) x 10;
    if $flags[pos: 2]:
        change $code to $code + 50;
    ;if
    change $code to $code + to.i64(std::ls.len(lend $none));
    change $code to $code + to.i64($eights[pos: 4]);
    let $twice: i64 := to.i64($bs[pos: 2] x 2.0) ?? 0;
    change $code to $code + $twice;
    no_check my.exit(to.i32($code));
;MAIN
