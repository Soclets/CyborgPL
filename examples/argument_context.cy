# An argument takes its parameter's type as its context, as a declaration's value does (owner,
# 2026-09-29: a): show[0 minus.wrap 1] wraps at 8 bits, and list.append's value takes the element type.
proc show [$b: int.unsign.8]:
    std::print.shortest($b, newln);
end of proc show
fn wide [$w: int.unsign.32] -> int.unsign.32:
    ret $w;
end of fn wide
!MAIN:
    show(0 minus.wrap 1);
    let $v: int.unsign.64 = to.int.unsign.64(wide(2 shl 3));
    std::print.shortest($v, " ", wide(not.bit 0), newln);
    var $xs: std::ls[int.unsign.8] = [1,];
    std::ls.append(lend.mut $xs, value: 0 minus.wrap 2);
    std::print.shortest($xs[pos: 2], newln);
!end of MAIN
