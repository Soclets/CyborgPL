# A C enum (owner, 2026-09-27: a): `layout.C enum`, a number per case as the C header
# gives it, no data. It is a CyborgPL enum in every other way — `match` covers every
# case, and it is owned like every enum (2026-09-20) — and C-shaped, so `extern.C fn`
# can take and return it. A number C hands back that no case has fails `out_of_range`.
%layout.C enum Key:
    a = 65;
    b = 66;
    escape = 256;
end of %layout.C enum Key
%extern.C proc IsKeyDown [let $key: Key] -> bool;

fn name [let $k: Key] -> str.utf8:
    match $k:
        case a:
            ret "A";
        end of case a
        case b:
            ret "B";
        end of case b
        case escape:
            ret "Esc";
        end of case escape
    end of match
end of fn name

!MAIN:
    let $k := Key.escape;
    std::print.shortest(name(copy $k), " ", name(Key.a), newln);
!end of MAIN
