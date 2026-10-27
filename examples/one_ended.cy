# One-ended ranges: a side left out is the type's own limit (owner, 2026-10-12: `i32[min: 1]` is
# "at least 1"; until then `min` and `max` named the limits, `i32[1, max]`, 2026-09-23).
!MAIN:
    let $pos: integer.signed.32_bit[minimum: 1] = 1;
    let $score: integer.signed.32_bit[maximum: 100] = 50;
    let $small: integer.signed.8_bit[minimum: 1] = 120;
    let $whole: integer.unsigned.8_bit[minimum: 0, maximum: 255] = 200;          # the whole type: harmless, allowed (owner)
    let $also: integer.signed.32_bit = negative 7;                             # no bound at all: the type itself
    std::print.shortest($pos, " ", $score, " ", $small, " ", $whole, " ", $also, newline);
!end of MAIN
