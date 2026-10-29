# One lane of a vector is written like a list item (owner, 2026-09-29: a),
# as examples/simd_lanes.cpls has it: 'x' is 4, a u8 200 +~ 200 is 144,
# lane 3 set to 30 reads 30 and sums to 30 — and Precog, which joins a
# written lane into the vector's domain, must not fold it back to 0.
# 4 + 144 + 30 + 30 = 208.
%extern.C proc exit [let $code: i32];
MAIN:
    let $a: simd[i32, count: 4] := simd.fill(value: 4);
    let $x: i32 := $a[pos: 1];
    let $u: simd[u8, count: 16] := simd.fill(value: 200);
    let $w := $u plus.wrap $u;
    var $v: simd[i32, count: 4] = simd.fill(value: 0);
    change $v[pos: 3] to 30;
    let $y: i32 := $v[pos: 3];
    let $s: i32 := simd.sum($v);
    no_check my.exit($x + to.i32($w[pos: 16]) + $y + $s);
;MAIN
