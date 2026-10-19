# A whole number in another base is written with a word in front, as `exactly` is (owner, 2026-10-19: "(a)"):
# `binary`, `hex`, `octal`, with `_` grouping as in decimals. The same number either way: binary 1010 is 10.
MAIN {
    let 'flags': uint8 = binary 1010_0101;
    let 'mask': uint32 = hex FF00_00FF;
    let 'low': uint32 = hex ff;
    let 'mode': uint16 = octal 755;
    let 'n' := -hex 10;
    std::print.useful['flags', " ", 'mask', " ", 'low', " ", 'mode', " ", 'n', " ", 'flags' and.bit binary 1111, newln];
}
