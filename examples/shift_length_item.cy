# A list's length is a u64 and an item has the element type, so either gives a shift its width
# (check claude, seeds 5018 and 5054, 2026-10-07).
!MAIN:
    let $xs: std::ls[int.unsign.8] := [10, 20, 30,];
    std::print.shortest((std::ls.len(lend $xs) shl 1) greater_than 5, " ", ($xs[pos: 1] shr 2) eq 2, newln);
!end of MAIN
