# gen_round.py pair|real: round.X over f16/32/64 and %d32/64/128 values, whole and to places.
# pair: "A …" the oracle's built-in round, "B …" stdtest's (std/ copied); real: "A …" std/ through kayo.
import sys

mode = sys.argv[1]
values = ["2.5", "-2.5", "0.5", "-0.5", "1.5", "2.675", "-2.675", "0.125", "0.375", "1234.5678", "-1234.5678",
          "0.1", "0.0", "9.995", "99.5", "0.045", "123456.75", "-0.001"]
modes = ["near", "half_up", "floor", "ceil", "trunc"]
places = [0, 1, 2, 3, 6, 20]
out = ["MAIN:"]
sides = [("A", "std::", True)] + ([("B", "stdtest::", False)] if mode == "pair" else [])


def calls(tag, label, pre, builtin, name):
    for m in modes:
        out.append(f'    std::print.stored("{tag} {label} {m} ", {pre}round.{m}(${name}), newln);')
        for d in places:
            arg = f"places: {d}" if builtin else f"{d}"
            out.append(f'    std::print.stored("{tag} {label} {m} {d} ", {pre}round.{m}(${name}, {arg}), newln);')


k = 0
for t in ["f16", "f32", "f64"]:
    for v in values:
        if t == "f16" and v in ("1234.5678", "-1234.5678", "123456.75"):
            continue
        k += 1
        out.append(f"    let $v{k}: {t} := {v};")
        for tag, pre, builtin in sides:
            calls(tag, f"{t} {v}", pre, builtin, f"v{k}")
dvalues = ["2.5", "0.5", "1.5", "2.675", "0.125", "1234.5678", "1234.5000", "9.995", "99.5", "0.045", "2.50", "0.1", "0", "1234567"]
for t in ["d32", "d64", "d128"]:
    for v in dvalues + [("12345678901234.56" if t != "d32" else "12345.67")]:
        if t == "d32" and len(v.replace(".", "").lstrip("0")) > 7:
            continue
        for sign in ["", "-"]:
            k += 1
            # the decis are the Money limb's, so their type is marked (owner, 2026-10-26)
            out.append(f"    let $w{k}: %{t} := exactly {v};")
            name = f"w{k}"
            if sign:
                out.append(f"    let $n{k}: %{t} := -$w{k};")
                name = f"n{k}"
            for tag, pre, builtin in sides:
                calls(tag, f"{t} {sign}{v}", pre, builtin, name)
out.append(";MAIN")
print("\n".join(out))
