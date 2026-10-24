# A match arm's data name has the case's type (found by check claude, 2026-09-29): 'd' is an i32,
# so not.bit and times.wrap know their width, and 'b' is a u8, so plus.wrap wraps at 8 bits.
enum E1 { c1: i32, c2: u8, c3 }
MAIN {
    let $e := E1.c1(5);
    match $e {
        c1 $d { std::print.useful(not.bit $d, " ", $d times.wrap 1000000000, newln); }
        c2 $b { }
        c3 { }
    }
    let $f := E1.c2(250);
    let $w := match $f { c1 $d { is 0; } c2 $b { is to.i64($b plus.wrap 10); } c3 { is 1; } };
    std::print.useful($w, newln);
}
