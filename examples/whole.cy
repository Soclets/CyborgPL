# bin → int fails unless the bin is whole (decided 2026-09-21), or `out_of_range` when it is whole but
# too big for the integer (owner, 2026-09-30: a). Rounding is said first.
MAIN {
    let $x: f64 = 2.0;
    let $y: f64 = 2.7;
    let $a := to.i32($x) otherwise -1;                    # whole: 2
    let $b := to.i32($y) otherwise -1;                    # not whole: fails → -1
    let $c := to.i32(std::round.near($y)) otherwise -1;   # said: 3
    let $d := to.i32(std::round.floor($y)) otherwise -1;  # said: 2
    let $e := to.i32(std::round.trunc(0.0 - $y)) otherwise -1;  # said: -2
    std::print.useful($a, " ", $b, " ", $c, " ", $d, " ", $e, newln);
    let $big: f64 = 10000000000.0;
    let $g := to.i32($big) otherwise -1;                 # whole but too big for an i32: out_of_range → -1
    std::print.useful($g, newln);
    let $f := to.i32($y) otherwise {
        not_whole { std::print.useful("2.7 is not whole", newln); std::exit(code: 0); }
        out_of_range { std::print.useful("too big", newln); std::exit(code: 0); }
    };
}
