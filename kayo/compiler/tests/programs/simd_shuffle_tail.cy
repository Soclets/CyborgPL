# `simd.shuffle` (owner, 2026-09-27: a) rearranges lanes by a fixed order;
# `simd.load.some` / `simd.store.some` touch only the first k lanes, so a
# list of any length is one vector loop (examples/simd_shuffle.cpls and
# examples/simd_tail.cpls, read out as an exit code).
%extern.C proc exit [$code: i32];

MAIN:
    let $ys: arr[i32] := [10, 20, 30, 40];
    let $v: simd[i32, count: 4] := simd.load(lend $ys, pos: 1);
    let $r := simd.shuffle($v, [4, 3, 2, 1]);
    let $pairs := simd.shuffle($v, [2, 1, 4, 3]);
    let $first := simd.shuffle($v, [1, 1, 1, 1]);
    var $out: arr[i32] = [0, 0, 0, 0];
    simd.store(lend.mut $out, pos: 1, value: $r);
    let $r1 := $out[pos: 1];                          # 40
    simd.store(lend.mut $out, pos: 1, value: $pairs);
    let $p2 := $out[pos: 2];                          # 10
    let $f := simd.sum($first);                      # 40

    var $xs: std::ls[i32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let $n := std::ls.len(lend $xs);
    var $acc: simd[i32, count: 4] = simd.fill(value: 0);
    var $i: u64 = 1;
    while $i <= $n:
        let $left := $n - $i + 1;
        if $left >= 4:
            let $w: simd[i32, count: 4] := simd.load(lend $xs, pos: $i);
            change $acc to $acc + $w;
        else:
            let $w: simd[i32, count: 4] := simd.load.some(lend $xs, pos: $i, count: $left);
            change $acc to $acc + $w;
        ;if
        change $i to $i + 4;
    ;while
    let $twice: simd[i32, count: 4] := simd.fill(value: 2);
    let $last: simd[i32, count: 4] := simd.load(lend $xs, pos: 7);
    simd.store.some(lend.mut $xs, pos: 9, count: 2, value: $last x $twice);
    let $x9 := $xs[pos: 9];                           # 14
    let $x10 := $xs[pos: 10];                         # 16
    let $none: simd[i32, count: 4] := simd.load.some(lend $xs, pos: 11, count: 0);
    # 40 + 10 + 40 + 55 + 14 + 16 + 0 = 175
    no_check my.exit($r1 + $p2 + $f + simd.sum($acc) + $x9 + $x10 + simd.sum($none));
;MAIN
