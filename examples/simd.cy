# simd[T, N] (design/simd.md, 2026-09-22): N plain values in one register, the ordinary
# operators lane by lane. The plain loop is the portable way; this is for when it is not enough.
fn is_ascii [lent $bytes: arr[u8]] -> bool:
    var $acc: simd[u8, count: 16] = simd.fill(value: 0);
    for $i [from: 1, to: arr.len(lend $bytes), by: 16]:
        change $acc to $acc or.bit simd.load(lend $bytes, pos: $i);
    ;for
    ret simd.max($acc) < 128;
;fn is_ascii

MAIN:
    let $plain: arr[u8] := [72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33, 33, 33, 33, 33];
    let $thai: arr[u8] := [72, 101, 108, 108, 111, 32, 224, 184, 129, 108, 100, 33, 33, 33, 33, 33];
    std::print(is_ascii(lend $plain), " ", is_ascii(lend $thai), newln);
    let $a: simd[i32, count: 4] := simd.fill(value: 10);
    let $b: simd[i32, count: 4] := simd.fill(value: 3);
    std::print($a + $b, " ", simd.sum($a), " ", simd.max($a x $b), newln);
    let $m := $a > $b;                       # a simd[bool, count 4], one answer per lane
    std::print(simd.all($m), " ", simd.any($b > $a), newln);
    var $out: arr[i32] = [0, 0, 0, 0, 0, 0];
    simd.store(lend.mut $out, pos: 2, value: $a + $b);
    std::print($out, newln);
;MAIN
