# A vector's lane is read like a list's item, 'v'[1], 1-based; the wrapping forms work lane by lane at
# the element's size (simd.md; found by check claude, 2026-09-29). A lane is written like a list item,
# `change 'v'[3] = 30;` (owner, 2026-09-29: a).
MAIN {
    let 'a': simd[int32, 4] = simd.fill[value 4];
    let 'x': int32 = 'a'[pos 1];
    let 'u': simd[uint8, 16] = simd.fill[value 200];
    let 'w' := 'u' plus.wrap 'u';
    std::print.stored['x', " ", 'w'[pos 1], " ", 'w'[pos 16], newln];
    let mut 'v': simd[int32, 4] = simd.fill[value 0];
    change 'v'[pos 3] = 30;
    std::print.stored['v'[pos 3], " ", simd.sum['v'], newln];
}
