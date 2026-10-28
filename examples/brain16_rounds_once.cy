# A number into a brain16 rounds once, to the nearest brain16 (types.md). 25262379159781017 lies between
# 179 x 2^47 and 180 x 2^47, just below half way, so it rounds down. Through f32 first it would land
# exactly half way and tie up (check claude, 2026-10-15).
MAIN:
    let $v: i64 := 25262379159781017;
    let $w := to.%bf16($v);
    let $l: %bf16 := 25262379159781017.0;
    std::print.stored($w, newln);
    std::print.stored($l, newln);
    # a f64 into a brain16 rounds once too: 1 + 2^-8 + 2^-30 is nearest 1 + 2^-7, where 32 bits
    # would drop the 2^-30 and tie down to 1 (compiler claude's case)
    let $x: f64 := 1.0039062509313226;
    std::print.stored(to.%bf16($x), newln);
    # a decimal rounds from its exact value: just below the half step 1.01171875 rounds down, though as
    # a f64 it would be the half step itself and tie up (check claude, 2026-10-15); the half step ties to even
    let $below: %bf16 := 1.01171874999999999999;
    let $tie: %bf16 := 1.01171875;
    let $text := to.%bf16("1.01171874999999999999") ?? 1.0;
    std::print.stored($below, " ", $tie, " ", $text, newln);
;MAIN
