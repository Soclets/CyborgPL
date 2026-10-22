# Finding 2026-10-22 (the sweep design claude asked for after seed 206): a deci has no -0, whatever made the
# zero (owner, 2026-10-07; oracle e409892c, examples/deci_no_minus_zero.cy). The oracle prints 0 in every case
# below; kayo prints -0 for each, in d32, d64 and d128 alike: negation, a zero times a negative either way
# round, a zero div.near a negative, mod.floor by a negative, a negated mod.floor/ceil/trunc, a conversion
# of a negated zero to a narrower or wider deci, the text "-0" / "-0.0" read as a deci, a percent made from a
# negated deci zero, and a negated zero under min_places. Rounding to zero and to.stored / to.useful of an
# f64 -0.0 give 0 in both.
MAIN {
    let 'z32': d32 = 0.0;
    let 'z': d64 = 0.0;
    let 'z128': d128 = 0.0;
    let 'n': d64 = -1.7;
    let 'k': d64 = -3.0;
    let 'f': d64 = -5.0;
    let 'g': d64 = 2.5;
    let 'h': d64 = -2.5;
    let 'p0' := exactly 0%;
    let 'w': d64[min_places: 3] = -'z';
    let 't' := to.d64["-0.0"] otherwise 'g';
    let 'c' := to.d32[-'z'] otherwise 'z32';
    std::print.stored[-'z32', " ", -'z', " ", -'z128', " ", 'z' x 'n', " ", 'n' x 'z', " ", 'z' div.near 'k', newln];
    std::print.stored[5.0 mod.floor 'h', " ", -('f' mod.floor 'g'), " ", -('f' mod.ceil 'g'), " ", -('f' mod.trunc 'g'), newln];
    std::print.stored['c', " ", to.d128[-'z'], " ", 't', " ", -'p0', " ", to.percent[d64][-'z'], " ", 'w', newln];
    std::print.useful[-'z', " ", 'z' x 'n', " ", 'w', newln];
}
