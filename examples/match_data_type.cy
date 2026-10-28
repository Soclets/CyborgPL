# A match arm's data name has the case's type (found by check claude, 2026-09-29): 'd' is an i32,
# so not.bit and times.wrap know their width, and 'b' is a u8, so plus.wrap wraps at 8 bits.
union.tag E1:
    c1: i32;
    c2: u8;
    c3;
;union.tag E1
MAIN:
    let $e := E1.c1(5);
    match $e:
        case c1 $d:
            std::print(not.bit $d, " ", $d times.wrap 1000000000, newln);
        ;case c1
        case c2 $b:
        ;case c2
        case c3:
        ;case c3
    ;match
    let $f := E1.c2(250);
    let $w := match $f:
        case c1 $d:
            is 0;
        ;case c1
        case c2 $b:
            is to.i64($b plus.wrap 10);
        ;case c2
        case c3:
            is 1;
        ;case c3
    ;match;
    std::print($w, newln);
;MAIN
