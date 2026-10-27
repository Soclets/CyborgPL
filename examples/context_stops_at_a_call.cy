# The type a value is stored into reaches the arithmetic that makes it, but stops at a call:
# what `g(…)` returns says nothing about the arithmetic in its arguments (2026-09-23).
function g [$p: boolean] -> integer.unsigned.8_bit:
    return 0;
end of function g
!MAIN:
    let $x: integer.signed.64_bit = 0;
    let $v: integer.unsigned.8_bit = g(0 greater_than (negative 1) minus $x);       # the (-1) minus 'x' is i64 arithmetic, not u8
    let $w: integer.unsigned.32_bit = 2 shift_left 3;                # but a bare literal still takes its width from here
    std::print.shortest($v, " ", $w, newline);
!end of MAIN
