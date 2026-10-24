# Finding 2026-10-22 (generator, seed 206): a deci has no -0 (owner, 2026-10-07; oracle b72c73ed, a zero
# deci or percent negated is 0, examples/negate_zero_deci.cy). kayo negates a zero d64 to -0: here
# "0.0 0.00 -0.00 -0.0"; the oracle "0.0 0.00 0.00 0.0". Seen beside it, not a difference: a zero d64
# times a negative (`'z' x (-1.7)`) prints "-0.00" in both engines.
MAIN {
    let 'a': d64 = -5.0;
    let 'm': d64 = 'a' mod.ceil 2.5;
    let 't': d64 = 'm' x 1.7;
    let 'z': d64 = 0.0;
    std::print.stored('m', " ", 't', " ", -'t', " ", -'z', newln);
}
