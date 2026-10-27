# A call's return type gives a shift its width (operators.md, 2026-09-23: a): `g()` is an i8,
# so `g() shl 1` is an 8-bit shift (check claude's seeds 55 and 123, 2026-10-06).
fn g [] -> int.sign.8:
    ret 10;
end of fn g
!MAIN:
    if 3 gt (g() shl 1):
        std::print.shortest("yes", newln);
    end of if
    else:
        std::print.shortest("no", newln);
    end of else
!end of MAIN
