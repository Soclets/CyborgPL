# A vector's lane is read like a list's item, 'v'[1], 1-based; the wrapping forms work lane by lane at
# the element's size (simd.md; found by check claude, 2026-09-29). A lane is written like a list item,
# `change $v[3] = 30;` (owner, 2026-09-29: a).
!MAIN:
    let $a: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 4);
    let $x: integer.signed.32_bit = $a[position: 1];
    let $u: simd[integer.unsigned.8_bit, count: 16] = simd.fill(value: 200);
    let $w := $u plus.wrap $u;
    std::print.stored($x, " ", $w[position: 1], " ", $w[position: 16], newline);
    variable $v: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 0);
    change $v[position: 3] = 30;
    std::print.stored($v[position: 3], " ", simd.sum($v), newline);
!end of MAIN
