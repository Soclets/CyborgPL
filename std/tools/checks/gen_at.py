# gen_at.py pair|real: str.at.byte / at.point at every position from 1 to the length + 2.
# pair: lines "A …" from the oracle's built-in std, "B …" from stdtest (std/ copied). real: "A …" from std/, through kayo.
import sys

mode = sys.argv[1]
texts = ["", "a", "abc", "h\xe9llo", "\xe9\U0001f34e\xe9", "\U0001f34e", "a\U0001f34eb",
         "\u0e44\u0e17\u0e22", "x\u0301y"]


def q(s):
    return "str.empty" if s == "" else '"' + s + '"'


out = ["MAIN {"]
sides = [("A", "std", True)] + ([("B", "stdtest", False)] if mode == "pair" else [])
for k, t in enumerate(texts, 1):
    out.append(f"    let $s{k} := {q(t)};")
    n = len(t.encode()) + 2
    for tag, prefix, builtin in sides:
        for unit in ["byte", "point"]:
            pos = "pos: $i" if builtin else "$i"
            out.append(f"    for $i: u64 [from: 1, to: {n}, by: 1] {{")
            out.append(f'        let $v := {prefix}::str.at.{unit}(lend $s{k}, {pos}) otherwise {{ out_of_range {{ std::print.stored("{tag} {unit} {k} ", $i, " out_of_range", newln); continue; }} }};')
            out.append(f'        std::print.stored("{tag} {unit} {k} ", $i, " ", $v, newln);')
            out.append("    }")
out.append("}")
print("\n".join(out))
