# The leftover end of a SIMD loop (owner, 2026-09-27: a): `simd.load.some` loads k items and
# leaves the other lanes zero; `simd.store.some` writes only the first k lanes. So a list of
# any length is one vector loop. k is 0 to N, and the k items must be in the list (a check
# site the prover proves, as for `simd.load`).
!MAIN:
    var $xs: std::ls[int.sign.32] := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let $n := std::ls.len(lend $xs);
    var $acc: simd[int.sign.32, count: 4] := simd.fill(value: 0);
    var $i: int.unsign.64 := 1;
    while $i at_most $n:
        let $left := $n minus $i plus 1;
        if $left at_least 4:
            let $v: simd[int.sign.32, count: 4] := simd.load(lend $xs, pos: $i);
            change $acc to $acc plus $v;
        end of if
        else:
            let $v: simd[int.sign.32, count: 4] := simd.load.some(lend $xs, pos: $i, count: $left);
            change $acc to $acc plus $v;
        end of else
        change $i to $i plus 4;
    end of while
    let $twice: simd[int.sign.32, count: 4] := simd.fill(value: 2);
    let $last: simd[int.sign.32, count: 4] := simd.load(lend $xs, pos: 7);
    simd.store.some(lend.mut $xs, pos: 9, count: 2, value: $last times $twice);
    std::print.shortest(simd.sum($acc), " ", $xs, newln);
!end of MAIN
