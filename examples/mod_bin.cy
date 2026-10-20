# `mod.floor` on a bin takes the divisor's sign, as on integers and decis (owner, 2026-10-02: a), so an
# angle stays 0 to 360; C's `fmod` (the dividend's sign) is `mod.trunc`, by name.
MAIN {
    let 'angle': f64 = -370.0;
    let 'turn': f64 = 360.0;
    let 'a': f64 = 7.5;
    let 'b': f64 = -2.0;
    let 'h': brain16 = -5.0;
    let 'k': brain16 = 3.0;
    std::print.useful['angle' mod.floor 'turn', " ", 'a' mod.floor 'b', " ", 'h' mod.floor 'k', newln];
    std::print.useful[('angle' mod.trunc 'turn'), " ", ('a' mod.trunc 'b'), newln];
    # a tiny negative would round up to the divisor itself; it is 0 instead, so the answer stays below 360
    let 'tiny': f64 = -0.00000000000000000001;
    std::print.useful['tiny' mod.floor 'turn', newln];
}
