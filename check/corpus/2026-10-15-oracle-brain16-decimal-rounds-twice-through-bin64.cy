# Finding 2026-10-15 (by hand, after cd32091f): a decimal literal or text into brain16 still rounds
# twice in the oracle, now through f64. 1.01171874999999999999 is just below the half step
# between 1.0078125 and 1.015625 (1.01171875), so the nearest brain16 is 1.0078125 (types.md:
# "rounds to the nearest brain16"; checked with exact fractions). Zomp prints 1.0078125 for both.
# The oracle prints 1.015625: as a f64 the number is exactly the half step, and ties to even go
# up. The exact half step, and a f64 (rounded as a f64 first, by: its own rule), agree.
MAIN {
    let 'a': brain16 = 1.01171874999999999999;
    std::print.stored['a', newln];
    let 'b' := to.brain16["1.01171874999999999999"] otherwise 1.0;
    std::print.stored['b', newln];
    let 'd': brain16 = 1.01171875;
    std::print.stored['d', newln];
}
