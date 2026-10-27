# simd[T, N] (design/simd.md, 2026-09-22): N plain values in one register, the ordinary
# operators lane by lane. The plain loop is the portable way; this is for when it is not enough.
fn is_ascii [lent $bytes: arr[int.unsign.8]] -> bool:
    var $acc: simd[int.unsign.8, count: 16] = simd.fill(value: 0);
    for $i [from: 1, to: arr.len(lend $bytes), by: 16]:
        change $acc to $acc or.bit simd.load(lend $bytes, pos: $i);
    end of for
    ret simd.max($acc) less_than 128;
end of fn is_ascii

!MAIN:
    let $plain: arr[int.unsign.8] := [72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33, 33, 33, 33, 33];
    let $thai: arr[int.unsign.8] := [72, 101, 108, 108, 111, 32, 224, 184, 129, 108, 100, 33, 33, 33, 33, 33];
    std::print.shortest(is_ascii(lend $plain), " ", is_ascii(lend $thai), newln);
    let $a: simd[int.sign.32, count: 4] := simd.fill(value: 10);
    let $b: simd[int.sign.32, count: 4] := simd.fill(value: 3);
    std::print.shortest($a plus $b, " ", simd.sum($a), " ", simd.max($a times $b), newln);
    let $m := $a greater_than $b;                       # a simd[bool, count 4], one answer per lane
    std::print.shortest(simd.all($m), " ", simd.any($b greater_than $a), newln);
    var $out: arr[int.sign.32] = [0, 0, 0, 0, 0, 0];
    simd.store(lend.mut $out, pos: 2, value: $a plus $b);
    std::print.shortest($out, newln);
!end of MAIN
