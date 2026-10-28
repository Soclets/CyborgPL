# A list's length is a u64 and an item has the element type, so either gives a shift its width
# (check claude, seeds 5018 and 5054, 2026-10-07).
MAIN:
    let $xs: std::ls[u8] := [10, 20, 30,];
    std::println((std::ls.len(lend $xs) shl 1) > 5, " ", ($xs[pos: 1] shr 2) == 2);
;MAIN
