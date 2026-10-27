# One-ended ranges: a side left out is the type's own limit (owner, 2026-10-12: `i32[min: 1]` is
# "at least 1"; until then `min` and `max` named the limits, `i32[1, max]`, 2026-09-23).
!MAIN:
    let $pos: int.sign.32[min: 1] = 1;
    let $score: int.sign.32[max: 100] = 50;
    let $small: int.sign.8[min: 1] = 120;
    let $whole: int.unsign.8[min: 0, max: 255] = 200;          # the whole type: harmless, allowed (owner)
    let $also: int.sign.32 = neg 7;                             # no bound at all: the type itself
    std::print.shortest($pos, " ", $score, " ", $small, " ", $whole, " ", $also, newln);
!end of MAIN
