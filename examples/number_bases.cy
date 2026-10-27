# A whole number in another base is written with a word in front, as `exactly` is (owner, 2026-10-19: "(a)"):
# `binary`, `hex`, `octal`, with `_` grouping as in decimals. The same number either way: bin 1010 is 10.
!MAIN:
    let $flags: int.unsign.8 = bin 1010_0101;
    let $mask: int.unsign.32 = hex FF00_00FF;
    let $low: int.unsign.32 = hex ff;
    let $mode: int.unsign.16 = oct 755;
    let $n := neg hex 10;
    std::print.shortest($flags, " ", $mask, " ", $low, " ", $mode, " ", $n, " ", $flags and.bit bin 1111, newln);
!end of MAIN
