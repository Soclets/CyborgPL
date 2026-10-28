# A list literal's items take the list's element type as their context, as a declaration's value
# does (owner, 2026-09-29: a): 0 minus.wrap 1 in a list[u8] wraps at 8 bits, 2 shl 3 in a list[u32]
# is a 32-bit shift, and bare numbers alone are still exact.
MAIN:
    let $i: u8 := 5;
    let $xs: std::ls[u8] := [0 minus.wrap 1, 0 minus.wrap $i, 250 plus.wrap 10];
    let $ys: std::ls[u32] := [2 shl 3, not.bit 0];
    let $zs: std::ls[std::ls[u8]] := [[0 minus.wrap 2,], [1,]];
    let $big: std::ls[u64] := [9692729768110883998 - 3166240359165959856,];
    let $inner := copy $zs[pos: 1];
    std::println($xs[pos: 1], " ", $xs[pos: 2], " ", $xs[pos: 3], " ", $ys[pos: 1], " ", $ys[pos: 2], " ", $inner[pos: 1], " ", $big[pos: 1]);
;MAIN
