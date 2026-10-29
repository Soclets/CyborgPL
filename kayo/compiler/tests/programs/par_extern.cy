# A loop that calls C stays in order by itself; `for para` runs it in
# slices anyway, the call's `nocheck` being the writer's word that the C
# is safe on many cores (owner, 2026-09-30). `labs` is: the sum of |i - 12345|
# mod 256 for i in 1 to 100000, as both loops give: 40 (worked out by hand;
# the oracle cannot link C).
%extern.C proc exit [let $code: i32];
%extern.C proc labs [let $n: i64] -> i64;
MAIN:
    let $n: i64 := 100000;
    var $a: std::ls[i64] = std::ls.fill(value: 0, count: $n);
    var $b: std::ls[i64] = std::ls.fill(value: 0, count: $n);
    for %para $i: i64 [from: 1, to: $n, by: 1]:
        no_check let $v := labs($i - 12345);
        change $a[pos: $i] to $v;
    ;for
    for $i: i64 [from: 1, to: $n, by: 1]:
        no_check let $v := labs($i - 12345);
        change $b[pos: $i] to $v;
    ;for
    var $same: i64 = 0;
    var $s: i64 = 0;
    for $k: i64 [from: 1, to: $n, by: 1]:
        if $a[pos: $k] == $b[pos: $k]:
            change $same to $same + 1;
        ;if
        change $s to ($s + $a[pos: $k]) mod.floor 256;
    ;for
    if $same != $n:
        no_check my.exit(1);
    ;if
    no_check my.exit(to.i32($s));
;MAIN
