# A typed literal: `5: u8` (owner, 2026-10-17: "yea", and "(a)": no brackets). The `:` covers only the
# literal right before it, as it covers only the name in `'x': u8`; a variable is converted with `to.…[…]`.
MAIN {
    # a shift's count gives nothing inside it a width, so the inner shift says its own
    let 'r': i64 = to.i64[1] shl (5: u8 shr 1);
    std::print.useful['r', newln];
    let 'a' := 200: u8;
    std::print.useful['a' + 55, newln];
    let 'n' := -128: i8;
    std::print.useful['n', newln];
    std::print.useful[3: u8 x 2, newln];
    let 'd' := exactly 0.1: d64;
    std::print.useful['d' + exactly 0.2, newln];
    let 'small' := exactly 2.5: d32;
    std::print.useful['small', newln];
    let 'p' := 5: u64[min: 1];
    std::print.useful['p', newln];
}
