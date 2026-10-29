# A C enum and a C struct with a ranged field cross to real C (owner,
# 2026-09-27: a): the enum is C's int, `div_t` comes back packed in one
# register as C returns it, and what C hands back is checked at the door —
# toupper['2'] is no case, 50 is past quot's range, and both take `otherwise`.
# The oracle cannot link C, so the answer is worked out by hand:
# 1000 + 200 + 30 + (3 + 1) + 9 = 1243, which exits as 1243 mod 256 = 219.
%extern.C proc exit [$code: i32];
%layout.C enum Key:
    a = 65; b = 66; escape = 256;
;%layout.C enum Key
%extern.C proc abs [$k: Key] -> i32;
%extern.C proc toupper [$c: i32] -> Key;
%layout.C rec Div:
    var quot: i32[min: 0, max: 10]; var rem: i32;
;%layout.C rec Div
%extern.C proc divide [$n: i32, $d: i32] -> Div;

fn code [$k: Key] -> i32:
    match $k:
        case a:
            ret 1;
        ;case a
        case b:
            ret 2;
        ;case b
        case escape:
            ret 3;
        ;case escape
    ;match
;fn code

MAIN:
    no_check let $n: i32 := abs(Key.escape);                        # 256
    no_check let $up: Key := toupper(97) ?? Key.escape;        # a
    no_check let $same: Key := toupper(66) ?? Key.escape;      # b
    no_check let $none: Key := toupper(50) ?? Key.escape;      # '2' is no case: escape
    no_check let $q: Div := divide(7, 2) ?? Div[quot = 0, rem = 9];
    no_check let $big: Div := divide(100, 2) ?? Div[quot = 0, rem = 9];
    no_check my.exit(($n - 256) + code($up) x 1000 + code($same) x 100 + code($none) x 10 + $q.quot + $q.rem + $big.rem);
;MAIN
