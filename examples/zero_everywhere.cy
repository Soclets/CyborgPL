# No bin or brain16 result is ever -0 (owner, 2026-10-07: "`a`"): a zero times a negative, a
# negated zero, a product too small to hold, are all plain 0. Each operator's `.ieee` form keeps
# IEEE's -0 where it is wanted ("yea"), and `neg.ieee` is IEEE's negation ("yea"): `-.ieee 'x'`
# "defeats the purpose of a symbol". Every operator that can make a -0 has its `.ieee` form, `mod`
# and `pow` included ("yea").
MAIN {
    let 'z': bin64 = 0.0;
    let 'n': bin64 = -2.0;
    let 'e30': bin64 = 0.000000000000000000000000000001;
    let 'e60': bin64 = 'e30' times 'e30';
    let 'e120': bin64 = 'e60' times 'e60';
    let 'tiny': bin64 = 'e120' times 'e120';
    std::print.useful['z' times 'n', " ", 'z' times.ieee 'n', newln];
    std::print.useful[-'z', " ", neg.ieee 'z', newln];
    std::print.useful['tiny' times (-'tiny'), " ", 'tiny' times.ieee (-'tiny'), newln];
    std::print.useful['z' div.near 'n', " ", 'z' div.near.ieee 'n', newln];
    let 'm': bin64 = neg.ieee 'z';
    std::print.useful['m' plus 'm', " ", 'm' plus.ieee 'm', " ", 'm' minus 'z', " ", 'm' minus.ieee 'z', newln];
    let 'b': brain16 = 0.0;
    let 'c': brain16 = -3.0;
    std::print.useful['b' times 'c', " ", 'b' times.ieee 'c', " ", -'b', " ", neg.ieee 'b', newln];
    let 'w': bin32 = 'e30' times 'e30' times (-1.0);
    std::print.useful['w', " ", 3.0 mod.floor (-3.0), " ", 3.0 mod.floor.ieee (-3.0), " ", (-3.0) mod.trunc 3.0, " ", (-3.0) mod.trunc.ieee 3.0, newln];
    # `mod` and `pow` keep it too (owner, 2026-10-07: "yea")
    std::print.useful[(-'e120') pow 3, " ", (-'e120') pow.ieee 3, " ", (-'e120') std::pow.ieee 3, newln];
}
