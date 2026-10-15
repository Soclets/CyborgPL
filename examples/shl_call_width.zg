# A call's return type gives a shift its width (operators.md, 2026-09-23: a): `g[]` is an int8,
# so `g[] shl 1` is an 8-bit shift (check claude's seeds 55 and 123, 2026-10-06).
fn g [] -> int8 { return 10; }
MAIN {
    if 3 > (g[] shl 1) { std::print.useful["yes", newln]; } else { std::print.useful["no", newln]; }
}
