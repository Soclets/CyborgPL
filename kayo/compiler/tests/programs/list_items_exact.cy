# A list literal's bare-number items are worked out exactly and checked where
# they land, like any typed store (owner, dc83c05; check claude, 2026-09-29,
# seed 197: an item was worked out at 64 bits, and 9692729768110883998 did
# not fit). 9692729768110883998 - 3166240359165959856 = 6526489408944924142,
# and (100 + 100) - 150 = 50 in a u8: 42 + 50 = 92 (the oracle agrees).
%extern.C proc exit [let $code: i32];
MAIN:
    let $big: std::ls[u64] := [9692729768110883998 - 3166240359165959856, 9692729768110883998 - 1];
    let $small: std::ls[u8] := [(100 + 100) - 150, 7];
    var $k: i32 = 0;
    if $big[pos: 1] == 6526489408944924142:
        change $k to 42;
    ;if
    no_check my.exit($k + to.i32($small[pos: 1]));
;MAIN
