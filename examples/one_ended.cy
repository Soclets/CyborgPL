# One-ended ranges: a side left out is the type's own limit (owner, 2026-10-12: `i32[min: 1]` is
# "at least 1"; until then `min` and `max` named the limits, `i32[1, max]`, 2026-09-23).
MAIN:
    let $pos: i32[min: 1] := 1;
    let $score: i32[max: 100] := 50;
    let $small: i8[min: 1] := 120;
    let $whole: u8[min: 0, max: 255] := 200;          # the whole type: harmless, allowed (owner)
    let $also: i32 := -7;                             # no bound at all: the type itself
    std::println($pos, " ", $score, " ", $small, " ", $whole, " ", $also);
;MAIN
