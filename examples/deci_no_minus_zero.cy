# A deci has no -0 (owner, 2026-10-07): a zero result is 0 whatever made it, negation, times a negative, a
# difference, a quotient (check claude, seed 206 and after).
MAIN {
    let 'a': d64 = -5.0;
    let 'm': d64 = 'a' mod.ceil 2.5;
    let 't': d64 = 'm' x 1.7;
    let 'z': d64 = 0.0;
    let 'n': d64 = -1.7;
    let 'k': d64 = -3.0;
    std::print.stored['m', " ", 't', " ", -'t', " ", -'z', " ", 'z' x 'n', " ", 'z' - 'z', " ", 'z' div.near 'k', newln];
}
