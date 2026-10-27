# A number from C becomes a C enum only through a check (owner, 2026-09-27). `to.Key(…)`
# converts it, failing `out_of_range` for a number no case has — here 302, a key a newer
# library added — so the program decides what an unknown key means instead of hitting
# undefined behaviour in the `match`. A callback, which C calls, takes the plain number.
%layout.C enumeration Key:
    a = 65;
    b = 66;
    escape = 256;
end of enumeration Key
function on_key [$code: integer.signed.32_bit] -> integer.signed.32_bit:
    let $k := to.Key($code) otherwise:
        !case out_of_range:
            return 0;
        !end of case out_of_range
    end of otherwise;
    match $k:
        case a:
            return 1;
        end of case a
        case b:
            return 2;
        end of case b
        case escape:
            return 3;
        end of case escape
    end of match
end of function on_key
!MAIN:
    std::print.shortest(on_key(66), " ", on_key(302), " ", on_key(256), newline);
!end of MAIN
