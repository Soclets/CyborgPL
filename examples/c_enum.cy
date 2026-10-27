# A C enum (owner, 2026-09-27: a): `layout.C enum`, a number per case as the C header
# gives it, no data. It is a CyborgPL enum in every other way — `match` covers every
# case, and it is owned like every enum (2026-09-20) — and C-shaped, so `extern.C fn`
# can take and return it. A number C hands back that no case has fails `out_of_range`.
%layout.C enumeration Key:
    a = 65;
    b = 66;
    escape = 256;
end of enumeration Key
%external.C procedure IsKeyDown [$key: Key] -> boolean;

function name [$k: Key] -> string.utf8:
    match $k:
        case a:
            return "A";
        end of case a
        case b:
            return "B";
        end of case b
        case escape:
            return "Esc";
        end of case escape
    end of match
end of function name

!MAIN:
    let $k := Key.escape;
    std::print.shortest(name(copy $k), " ", name(Key.a), newline);
!end of MAIN
