# The type a value is stored into reaches the arithmetic that makes it, but stops at a call:
# what `g(…)` returns says nothing about the arithmetic in its arguments (2026-09-23).
fn g [let $p: bool] -> u8:
    ret 0;
;fn g
MAIN:
    let $x: i64 := 0;
    let $v: u8 := g(0 > (-1) - $x);       # the (-1) minus 'x' is i64 arithmetic, not u8
    let $w: u32 := 2 shl 3;                # but a bare literal still takes its width from here
    std::println($v, " ", $w);
;MAIN
