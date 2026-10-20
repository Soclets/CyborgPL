# An argument takes its parameter's type as its context, as a declaration's value does (owner,
# 2026-09-29: a): show[0 minus.wrap 1] wraps at 8 bits, and list.append's value takes the element type.
fn show ['b': u8] { std::print.useful['b', newln]; }
fn wide ['w': u32] -> u32 { return 'w'; }
MAIN {
    show[0 minus.wrap 1];
    let 'v': u64 = to.u64[wide[2 shl 3]];
    std::print.useful['v', " ", wide[not.bit 0], newln];
    let mut 'xs': list[u8] = [1,];
    list.append[lend.mut 'xs', value: 0 minus.wrap 2];
    std::print.useful['xs'[pos: 2], newln];
}
