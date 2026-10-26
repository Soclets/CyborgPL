# A `list[bool]` used only by position is held as bits (owner, 2026-10-18:
# "Pack a `list[bool]` into bits"); the program cannot tell. Thirteen
# trues, the 3rd and 13th cleared, the 8th cleared and set again from
# values known only as it runs: 11. The primes to 100_000, by a sieve whose
# marking runs in parallel slices that share bytes: 9592. A list handed to
# a call stays bytes and still reads the same: 2 of 3 set. 11 + 9592 + 2 is
# 9605, and 9605 mod 256 is 133.
%extern.C fn exit [$code: i32];

fn trues [lent $xs: std::list[bool]] -> i64 {
    let mut $k: i64 = 0;
    for $i: i64 [from: 1, to: to.i64(std::list.length(lend $xs)), by: 1] {
        if $xs[pos: to.u64($i)] { change $k = $k plus 1; }
    }
    return $k;
}

MAIN {
    let mut $a: std::list[bool] = std::list.fill(value: true, count: 13);
    let $n: i64 = to.i64(std::list.length(lend $a));
    change $a[pos: 3] = false;
    change $a[pos: 13] = false;
    change $a[pos: 8] = $n equal_to 0;
    change $a[pos: 8] = $n equal_to 13;
    let mut $code: i64 = 0;
    for $i: i64 [from: 1, to: $n, by: 1] {
        if $a[pos: to.u64($i)] { change $code = $code plus 1; }
    }

    let $top: i64 = 100_000;
    let mut $comp: std::list[bool] = std::list.fill(value: false, count: $top plus 1);
    for $p: i64 [from: 2, to: $top, by: 1] {
        if not $comp[pos: to.u64($p plus 1)] {
            change $code = $code plus 1;
            for $m: i64 [from: $p times $p, to: $top, by: $p] {
                change $comp[pos: to.u64($m plus 1)] = true;
            }
        }
    }

    let mut $kept: std::list[bool] = std::list.fill(value: false, count: 3);
    change $kept[pos: 1] = true;
    change $kept[pos: 3] = true;
    change $code = $code plus trues(lend $kept);
    no_check my.exit(to.i32($code mod.floor 256));
}
