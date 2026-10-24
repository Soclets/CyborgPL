# A list literal's bare-number items are worked out exactly and checked where
# they land, like any typed store (owner, dc83c05; check claude, 2026-09-29,
# seed 197: an item was worked out at 64 bits, and 9692729768110883998 did
# not fit). 9692729768110883998 - 3166240359165959856 = 6526489408944924142,
# and (100 + 100) - 150 = 50 in a u8: 42 + 50 = 92 (the oracle agrees).
extern.C fn exit [$code: i32];
MAIN {
    let $big: std::list[u64] = [9692729768110883998 minus 3166240359165959856, 9692729768110883998 minus 1];
    let $small: std::list[u8] = [(100 plus 100) minus 150, 7];
    let mut $k: i32 = 0;
    if $big[pos: 1] equal_to 6526489408944924142 { change $k = 42; }
    no_check my.exit($k plus to.i32($small[pos: 1]));
}
