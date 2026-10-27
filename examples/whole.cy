# bin → int fails unless the bin is whole (decided 2026-09-21), or `out_of_range` when it is whole but
# too big for the integer (owner, 2026-09-30: a). Rounding is said first.
!MAIN:
    let $x: float.binary.64_bit = 2.0;
    let $y: float.binary.64_bit = 2.7;
    let $a := to.integer.signed.32_bit($x) otherwise negative 1;                    # whole: 2
    let $b := to.integer.signed.32_bit($y) otherwise negative 1;                    # not whole: fails → -1
    let $c := to.integer.signed.32_bit(std::round.nearest($y)) otherwise negative 1;   # said: 3
    let $d := to.integer.signed.32_bit(std::round.floor($y)) otherwise negative 1;  # said: 2
    let $e := to.integer.signed.32_bit(std::round.truncate(0.0 minus $y)) otherwise negative 1;  # said: -2
    std::print.shortest($a, " ", $b, " ", $c, " ", $d, " ", $e, newline);
    let $big: float.binary.64_bit = 10000000000.0;
    let $g := to.integer.signed.32_bit($big) otherwise negative 1;                 # whole but too big for an i32: out_of_range → -1
    std::print.shortest($g, newline);
    let $f := to.integer.signed.32_bit($y) otherwise:
        case not_whole:
            std::print.shortest("2.7 is not whole", newline); std::exit(code: 0);
        end of case not_whole
        case out_of_range:
            std::print.shortest("too big", newline); std::exit(code: 0);
        end of case out_of_range
    end of otherwise;
!end of MAIN
