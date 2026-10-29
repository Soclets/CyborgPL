# A number from C becomes a C enum only through a check (owner, 2026-09-27). `to.Key(…)`
# converts it, failing `out_of_range` for a number no case has — here 302, a key a newer
# library added — so the program decides what an unknown key means instead of hitting
# undefined behaviour in the `match`. A callback, which C calls, takes the plain number.
%layout.C enum Key:
    a = 65;
    b = 66;
    escape = 256;
;%layout.C enum Key
fn on_key [$code: i32] -> i32:
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
    std::print(on_key(66), " ", on_key(302), " ", on_key(256), newln);
;MAIN
