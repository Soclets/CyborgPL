# Division and remainder: `div.floor` and `mod.floor` by default, `div.trunc` and `mod.trunc` by name.
MAIN {
    # floor division and the mathematician's mod.floor (decided 2026-09-20)
    std::print.useful[7 div.floor 2, " ", 7 mod.floor 2, newln];
    std::print.useful[-7 div.floor 2, " ", -7 mod.floor 2, newln];        # -4 and 1: the clock never goes negative
    std::print.useful[7 div.floor -2, " ", 7 mod.floor -2, newln];        # -4 and -1: the result has the divisor's sign
    let 'hour' := -1;
    std::print.useful[('hour' mod.floor 12) + 1, newln];         # an hour before 1 o'clock is 12
    # hardware truncation, by name, when a C library's answer must match bit for bit
    std::print.useful[(-7 div.trunc 2), " ", (-7 mod.trunc 2), newln];
}
