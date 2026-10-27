# The type a value is stored into reaches the arithmetic that makes it, but stops at a call:
# what `g(…)` returns says nothing about the arithmetic in its arguments (2026-09-23).
fn g [let $p: bool] -> int.unsign.8:
    ret 0;
end of fn g
!MAIN:
    let $x: int.sign.64 = 0;
    let $v: int.unsign.8 = g(0 greater_than (neg 1) minus $x);       # the (-1) minus 'x' is i64 arithmetic, not u8
    let $w: int.unsign.32 = 2 shl 3;                # but a bare literal still takes its width from here
    std::print.shortest($v, " ", $w, newln);
!end of MAIN
