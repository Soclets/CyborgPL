# A call's return type gives a shift its width (operators.md, 2026-09-23: a): `g()` is an i8,
# so `g() shl 1` is an 8-bit shift (check claude's seeds 55 and 123, 2026-10-06).
function g [] -> integer.signed.8_bit:
    return 10;
end of function g
!MAIN:
    if 3 greater_than (g() shift_left 1):
        std::print.shortest("yes", newline);
    end of if
    else:
        std::print.shortest("no", newline);
    end of else
!end of MAIN
