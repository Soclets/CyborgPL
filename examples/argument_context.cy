# An argument takes its parameter's type as its context, as a declaration's value does (owner,
# 2026-09-29: a): show[0 minus.wrap 1] wraps at 8 bits, and list.append's value takes the element type.
proc show [let $b: u8]:
    std::println($b);
;proc show
fn wide [let $w: u32] -> u32:
    ret $w;
;fn wide
MAIN:
    show(0 minus.wrap 1);
    let $v: u64 := to.u64(wide(2 shl 3));
    std::println($v, " ", wide(not.bit 0));
    var $xs: std::ls[u8] = [1,];
    std::ls.append(lend.mut $xs, value: 0 minus.wrap 2);
    std::println($xs[pos: 2]);
;MAIN
