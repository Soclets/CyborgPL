# A number from C becomes a C enum only through a check (owner, 2026-09-27):
# `to.Key[…]` fails `out_of_range` for a number no case has — 302, a key a
# newer library added — so the program decides what it means
# (examples/c_enum_convert.cpls as an exit code: 2, 0, 3 → 203).
%extern.C proc exit [let $code: i32];
%layout.C enum Key:
    a = 65; b = 66; escape = 256;
;%layout.C enum Key
fn on_key [let $code: i32] -> i32:
    let $k := to.Key($code)-
    otherwise:
        case out_of_range:
            ret 0;
        ;case out_of_range
    ;otherwise;
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
;fn on_key
MAIN:
    no_check my.exit(on_key(66) x 100 + on_key(302) x 10 + on_key(256));
;MAIN
