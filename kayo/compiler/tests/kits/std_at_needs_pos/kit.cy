# std's `str.at` takes its position with `pos` (owner, 2026-10-08), even
# written in CyborgPL: a bare number is refused.

MAIN:
    let $h := "hi";
    let $i := std::str.at.byte(lend $h, 2) ?? 0;
    std::print($i, newln);
;MAIN
