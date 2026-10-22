# A list's length is a u64 and an item has the element type, so either gives a shift its width
# (check claude, seeds 5018 and 5054, 2026-10-07).
MAIN {
    let 'xs': std::list[u8] = [10, 20, 30,];
    std::print.useful[(std::list.length[lend 'xs'] shl 1) > 5, " ", ('xs'[pos: 1] shr 2) equal_to 2, newln];
}
