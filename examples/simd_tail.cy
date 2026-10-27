# The leftover end of a SIMD loop (owner, 2026-09-27: a): `simd.load.some` loads k items and
# leaves the other lanes zero; `simd.store.some` writes only the first k lanes. So a list of
# any length is one vector loop. k is 0 to N, and the k items must be in the list (a check
# site the prover proves, as for `simd.load`).
!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let $n := std::list.length(lend $xs);
    variable $acc: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 0);
    variable $i: integer.unsigned.64_bit = 1;
    while $i at_most $n:
        let $left := $n minus $i plus 1;
        if $left at_least 4:
            let $v: simd[integer.signed.32_bit, count: 4] = simd.load(lend $xs, position: $i);
            change $acc = $acc plus $v;
        end of if
        else:
            let $v: simd[integer.signed.32_bit, count: 4] = simd.load.some(lend $xs, position: $i, count: $left);
            change $acc = $acc plus $v;
        end of else
        change $i = $i plus 4;
    end of while
    let $twice: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 2);
    let $last: simd[integer.signed.32_bit, count: 4] = simd.load(lend $xs, position: 7);
    simd.store.some(lend.mutable $xs, position: 9, count: 2, value: $last times $twice);
    std::print.shortest(simd.sum($acc), " ", $xs, newline);
!end of MAIN
