# A list literal's items take the list's element type as their context, as a declaration's value
# does (owner, 2026-09-29: a): 0 minus.wrap 1 in a list[u8] wraps at 8 bits, 2 shl 3 in a list[u32]
# is a 32-bit shift, and bare numbers alone are still exact.
!MAIN:
    let $i: integer.unsigned.8_bit = 5;
    let $xs: std::list[integer.unsigned.8_bit] = [0 minus.wrap 1, 0 minus.wrap $i, 250 plus.wrap 10];
    let $ys: std::list[integer.unsigned.32_bit] = [2 shift_left 3, not.bit 0];
    let $zs: std::list[std::list[integer.unsigned.8_bit]] = [[0 minus.wrap 2,], [1,]];
    let $big: std::list[integer.unsigned.64_bit] = [9692729768110883998 minus 3166240359165959856,];
    let $inner := copy $zs[position: 1];
    std::print.shortest($xs[position: 1], " ", $xs[position: 2], " ", $xs[position: 3], " ", $ys[position: 1], " ", $ys[position: 2], " ", $inner[position: 1], " ", $big[position: 1], newline);
!end of MAIN
