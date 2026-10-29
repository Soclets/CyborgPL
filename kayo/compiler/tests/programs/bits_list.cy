# A `list[bool]` used only by position is held as bits (owner, 2026-10-18:
# "Pack a `list[bool]` into bits"); the program cannot tell. Thirteen
# trues, the 3rd and 13th cleared, the 8th cleared and set again from
# values known only as it runs: 11. The primes to 100_000, by a sieve whose
# marking runs in parallel slices that share bytes: 9592. A list handed to
# a call stays bytes and still reads the same: 2 of 3 set. 11 + 9592 + 2 is
# 9605, and 9605 mod 256 is 133.
%extern.C proc exit [let $code: i32];

fn trues [lent $xs: std::ls[bool]] -> i64:
    var $k: i64 = 0;
    for $i: i64 [from: 1, to: to.i64(std::ls.len(lend $xs)), by: 1]:
        if $xs[pos: to.u64($i)]:
            change $k to $k + 1;
        ;if
    ;for
    ret $k;
;fn trues

MAIN:
    var $a: std::ls[bool] = std::ls.fill(value: true, count: 13);
    let $n: i64 := to.i64(std::ls.len(lend $a));
    change $a[pos: 3] to false;
    change $a[pos: 13] to false;
    change $a[pos: 8] to $n == 0;
    change $a[pos: 8] to $n == 13;
    var $code: i64 = 0;
    for $i: i64 [from: 1, to: $n, by: 1]:
        if $a[pos: to.u64($i)]:
            change $code to $code + 1;
        ;if
    ;for

    let $top: i64 := 100_000;
    var $comp: std::ls[bool] = std::ls.fill(value: false, count: $top + 1);
    for $p: i64 [from: 2, to: $top, by: 1]:
        if not $comp[pos: to.u64($p + 1)]:
            change $code to $code + 1;
            for $m: i64 [from: $p x $p, to: $top, by: $p]:
                change $comp[pos: to.u64($m + 1)] to true;
            ;for
        ;if
    ;for

    var $kept: std::ls[bool] = std::ls.fill(value: false, count: 3);
    change $kept[pos: 1] to true;
    change $kept[pos: 3] to true;
    change $code to $code + trues(lend $kept);
    no_check my.exit(to.i32($code mod.floor 256));
;MAIN
