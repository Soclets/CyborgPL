# No bin or brain16 result is ever -0 (owner, 2026-10-07: "`a`"): a zero times a negative, a
# negated zero, a product too small to hold, are all plain 0. Each operator's `.ieee` form keeps
# IEEE's -0 where it is wanted ("yea"), and `neg.ieee` (for a while `negate.ieee`) is IEEE's negation ("yea"): `-.ieee $x`
# "defeats the purpose of a symbol". Every operator that can make a -0 has its `.ieee` form, `mod`
# and `^` included ("yea").
MAIN:
    let $z: f64 := 0.0;
    let $n: f64 := -2.0;
    let $e30: f64 := 0.000000000000000000000000000001;
    let $e60: f64 := $e30 x $e30;
    let $e120: f64 := $e60 x $e60;
    let $tiny: f64 := $e120 x $e120;
    std::print($z x $n, " ", $z times.ieee $n, newln);
    std::print(-$z, " ", neg.ieee $z, newln);
    std::print($tiny x (-$tiny), " ", $tiny times.ieee (-$tiny), newln);
    std::print($z div.near $n, " ", $z div.near.ieee $n, newln);
    let $m: f64 := neg.ieee $z;
    std::print($m + $m, " ", $m plus.ieee $m, " ", $m - $z, " ", $m minus.ieee $z, newln);
    let $b: %bf16 := 0.0;
    let $c: %bf16 := -3.0;
    std::print($b x $c, " ", $b times.ieee $c, " ", -$b, " ", neg.ieee $b, newln);
    let $w: f32 := $e30 x $e30 x (-1.0);
    std::print($w, " ", 3.0 mod.floor (-3.0), " ", 3.0 mod.floor.ieee (-3.0), " ", (-3.0) mod.trunc 3.0, " ", (-3.0) mod.trunc.ieee 3.0, newln);
    # `mod` and `^` keep it too (owner, 2026-10-07: "yea")
    std::print((-$e120) ^ 3, " ", (-$e120) pow.ieee 3, " ", (-$e120) std::pow.ieee 3, newln);
;MAIN
