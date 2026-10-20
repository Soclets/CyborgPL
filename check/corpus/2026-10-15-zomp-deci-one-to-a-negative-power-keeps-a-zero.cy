# Finding 2026-10-15 (check run, seed 2445, cut down by hand): a deci 1 or -1 to a negative power
# prints with a trailing zero in Zomp. types.md: "a negative exponent is one over the power", and
# trailing zeros are kept "as IEEE does", so 1 pow (-1) is 1 div 1, exponent 0 - 0: 1. The oracle
# prints 1, 1, -1. Zomp prints 1.0, 1.0, -1.0, though its own 1 div.near 1 and 1 pow 2 print 1,
# and 2 pow (-1), 0.5 pow (-1) and 5 pow (-1) agree (0.5, 2, 0.2): only a base of 1 or -1.
MAIN {
    let 'a': d32 = to.d32[1] pow (-1);
    std::print.stored['a', newln];
    let 'b': d64 = to.d64[1] pow (-2);
    std::print.stored['b', newln];
    let 'c': d32 = to.d32[-1] pow (-1);
    std::print.stored['c', newln];
    let 'd': d32 = to.d32[1] div.near to.d32[1];
    std::print.stored['d', newln];
}
