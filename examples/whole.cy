# bin → int fails unless the bin is whole (decided 2026-09-21), or `out_of_range` when it is whole but
# too big for the integer (owner, 2026-09-30: a). Rounding is said first.
!MAIN:
    let $x: float.bin.64 = 2.0;
    let $y: float.bin.64 = 2.7;
    let $a := to.int.sign.32($x) otherwise neg 1;                    # whole: 2
    let $b := to.int.sign.32($y) otherwise neg 1;                    # not whole: fails → -1
    let $c := to.int.sign.32(std::round.near($y)) otherwise neg 1;   # said: 3
    let $d := to.int.sign.32(std::round.floor($y)) otherwise neg 1;  # said: 2
    let $e := to.int.sign.32(std::round.trunc(0.0 minus $y)) otherwise neg 1;  # said: -2
    std::print.shortest($a, " ", $b, " ", $c, " ", $d, " ", $e, newln);
    let $big: float.bin.64 = 10000000000.0;
    let $g := to.int.sign.32($big) otherwise neg 1;                 # whole but too big for an i32: out_of_range → -1
    std::print.shortest($g, newln);
    let $f := to.int.sign.32($y)-
    otherwise:
        case not_whole:
            std::print.shortest("2.7 is not whole", newln); std::exit(code: 0);
        end of case not_whole
        case out_of_range:
            std::print.shortest("too big", newln); std::exit(code: 0);
        end of case out_of_range
    end of otherwise;
!end of MAIN
