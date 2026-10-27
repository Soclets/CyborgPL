# A match arm's data name has the case's type (found by check claude, 2026-09-29): 'd' is an i32,
# so not.bit and times.wrap know their width, and 'b' is a u8, so plus.wrap wraps at 8 bits.
union.tag E1:
    c1: int.sign.32;
    c2: int.unsign.8;
    c3;
end of union.tag E1
!MAIN:
    let $e := E1.c1(5);
    match $e:
        case c1 $d:
            std::print.shortest(not.bit $d, " ", $d times.wrap 1000000000, newln);
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
            is to.int.sign.64($b plus.wrap 10);
        end of case c2
        case c3:
            is 1;
        end of case c3
    end of match;
    std::print.shortest($w, newln);
!end of MAIN
