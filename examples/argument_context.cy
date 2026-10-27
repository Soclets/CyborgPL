# An argument takes its parameter's type as its context, as a declaration's value does (owner,
# 2026-09-29: a): show[0 minus.wrap 1] wraps at 8 bits, and list.append's value takes the element type.
procedure show [$b: integer.unsigned.8_bit]:
    std::print.shortest($b, newline);
end of procedure show
function wide [$w: integer.unsigned.32_bit] -> integer.unsigned.32_bit:
    return $w;
end of function wide
!MAIN:
    show(0 minus.wrap 1);
    let $v: integer.unsigned.64_bit = to.integer.unsigned.64_bit(wide(2 shift_left 3));
    std::print.shortest($v, " ", wide(not.bit 0), newline);
    variable $xs: std::list[integer.unsigned.8_bit] = [1,];
    std::list.append(lend.mutable $xs, value: 0 minus.wrap 2);
    std::print.shortest($xs[position: 2], newline);
!end of MAIN
