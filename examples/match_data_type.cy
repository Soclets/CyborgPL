# A match arm's data name has the case's type (found by check claude, 2026-09-29): 'd' is an i32,
# so not.bit and times.wrap know their width, and 'b' is a u8, so plus.wrap wraps at 8 bits.
union.tagged E1:
    c1: integer.signed.32_bit; c2: integer.unsigned.8_bit; c3;
end of union.tagged E1
!MAIN:
    let $e := E1.c1(5);
    match $e:
        case c1 $d:
            std::print.shortest(not.bit $d, " ", $d times.wrap 1000000000, newline);
        end of case c1
        case c2 $b:
        end of case c2
        case c3:
        end of case c3
    end of match
    let $f := E1.c2(250);
    let $w := match $f:
        case c1 $d:
            is 0;
        end of case c1
        case c2 $b:
            is to.integer.signed.64_bit($b plus.wrap 10);
        end of case c2
        case c3:
            is 1;
        end of case c3
    end of match;
    std::print.shortest($w, newline);
!end of MAIN
