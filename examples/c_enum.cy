# A C enum (owner, 2026-09-27: a): `layout.C enum`, a number per case as the C header
# gives it, no data. It is a CyborgPL enum in every other way — `match` covers every
# case, and it is owned like every enum (2026-09-20) — and C-shaped, so `extern.C fn`
# can take and return it. A number C hands back that no case has fails `out_of_range`.
%layout.C enum Key:
    a = 65;
    b = 66;
    escape = 256;
;%layout.C enum Key
%extern.C proc IsKeyDown [let $key: Key] -> bool;

fn name [let $k: Key] -> str.utf8:
    match $k:
        case a:
            ret "A";
        ;case a
        case b:
            ret "B";
        ;case b
        case escape:
            ret "Esc";
        ;case escape
    ;match
;fn name

MAIN:
    let $k := Key.escape;
    std::println(name($k), " ", name(Key.a));
;MAIN
