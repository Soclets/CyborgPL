# A wide integer into a deci can fail `too_many_digits`, whatever its value, as any integer type wider
# than the deci's digits can (compiler claude, 2026-10-05: the oracle had said "cannot fail" past i128).
MAIN {
    let 'small': not_std.i300 = 5;
    let 'huge': not_std.i300 = 2 xx 200;
    let 'a' := to.d128['small'] otherwise 0.0;
    let 'b' := to.d128['huge'] otherwise 0.0;
    std::print.useful['a', " ", 'b', newln];
}
