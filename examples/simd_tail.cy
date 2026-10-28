# The leftover end of a SIMD loop (owner, 2026-09-27: a): `simd.load.some` loads k items and
# leaves the other lanes zero; `simd.store.some` writes only the first k lanes. So a list of
# any length is one vector loop. k is 0 to N, and the k items must be in the list (a check
# site the prover proves, as for `simd.load`).
MAIN:
    var $xs: std::ls[i32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let $n := std::ls.len(lend $xs);
    var $acc: simd[i32, count: 4] = simd.fill(value: 0);
    var $i: u64 = 1;
    while $i <= $n:
        let $left := $n - $i + 1;
        if $left >= 4:
            let $v: simd[i32, count: 4] := simd.load(lend $xs, pos: $i);
            change $acc += $v;
        else:
            let $v: simd[i32, count: 4] := simd.load.some(lend $xs, pos: $i, count: $left);
            change $acc += $v;
        ;if
        change $i += 4;
    ;while
    let $twice: simd[i32, count: 4] := simd.fill(value: 2);
    let $last: simd[i32, count: 4] := simd.load(lend $xs, pos: 7);
    simd.store.some(lend.mut $xs, pos: 9, count: 2, value: $last x $twice);
    std::println(simd.sum($acc), " ", $xs);
;MAIN
