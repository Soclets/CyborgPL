# simd[T, N] (design/simd.md, 2026-09-22): N plain values in one register, the ordinary
# operators lane by lane. The plain loop is the portable way; this is for when it is not enough.
function is_ascii [lent $bytes: array[integer.unsigned.8_bit]] -> boolean:
    variable $acc: simd[integer.unsigned.8_bit, count: 16] = simd.fill(value: 0);
    for $i [from: 1, to: array.length(lend $bytes), by: 16]:
        change $acc = $acc or.bit simd.load(lend $bytes, position: $i);
    end of for
    return simd.max($acc) less_than 128;
end of function is_ascii

!MAIN:
    let $plain: array[integer.unsigned.8_bit] = [72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33, 33, 33, 33, 33];
    let $thai: array[integer.unsigned.8_bit] = [72, 101, 108, 108, 111, 32, 224, 184, 129, 108, 100, 33, 33, 33, 33, 33];
    std::print.shortest(is_ascii(lend $plain), " ", is_ascii(lend $thai), newline);
    let $a: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 10);
    let $b: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 3);
    std::print.shortest($a plus $b, " ", simd.sum($a), " ", simd.max($a times $b), newline);
    let $m := $a greater_than $b;                       # a simd[bool, count 4], one answer per lane
    std::print.shortest(simd.all($m), " ", simd.any($b greater_than $a), newline);
    variable $out: array[integer.signed.32_bit] = [0, 0, 0, 0, 0, 0];
    simd.store(lend.mutable $out, position: 2, value: $a plus $b);
    std::print.shortest($out, newline);
!end of MAIN
