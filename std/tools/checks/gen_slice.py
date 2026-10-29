# gen_slice.py pair|real|python: str.at.cluster, str.slice.cluster and str.slice.point over every position pair.
# pair: "A …" from the oracle's built-in std (which has no slice.point), "B …" from stdtest (std/ copied).
# real: "A …" from std/ through kayo. python: what slice.point should give, by Python's own code points.
import sys

mode = sys.argv[1]
texts = ["", "a", "abc", "h\xe9llo", "\xe9\U0001f34e\xe9", "a\U0001f468\u200d\U0001f469\u200d\U0001f467b",
         "\u0e44\u0e17\u0e22", "x\u0301y", "\U0001f1f9\U0001f1ed\U0001f1ef\U0001f1f5", "e\u0301\u0301"]


def q(s):
    return "str.empty" if s == "" else '"' + s + '"'


def otherwise(indent, arm):
    """The otherwise below a call's line, its one arm out_of_range doing `arm`."""
    i = " " * indent
    return [f"{i}otherwise:", f"{i}    case out_of_range:", f"{i}        {arm}", f"{i}    ;case out_of_range", f"{i};otherwise;"]



if mode == "python":
    for k, t in enumerate(texts, 1):
        n = len(t) + 2
        for a in range(1, n + 1):
            for b in range(1, n + 1):
                print(f"slp {k} {a} {b} out_of_range" if b < a or b > len(t) else f"slp {k} {a} {b} [{t[a - 1:b]}]")
    sys.exit(0)

out = ["MAIN:"]
sides = [("A", "std", True)] + ([("B", "stdtest", False)] if mode == "pair" else [])
for k, t in enumerate(texts, 1):
    out.append(f"    let $s{k} := {q(t)};")
    n = len(t) + 2
    for tag, prefix, builtin in sides:
        posarg = "pos: $i" if builtin else "$i"
        out.append(f"    for $i: u64 [from: 1, to: {n}, by: 1]:")
        out.append(f"        let $v := {prefix}::str.at.cluster(lend $s{k}, {posarg})-")
        out += otherwise(8, f'std::print.stored("{tag} atg {k} ", $i, " out_of_range", newln); continue;')
        out.append(f'        std::print.stored("{tag} atg {k} ", $i, " [", $v, "]", newln);')
        out.append("    ;for")
        # The lines' tags are the old ones, g for a cluster (once a grapheme) and p for a point.
        units = [("cluster", "g")] + ([("point", "p")] if (mode == "real" or not builtin) else [])
        for unit, u in units:
            ab = "from: pos $a, to: pos $b" if builtin else "$a, $b"
            out.append(f"    for $a: u64 [from: 1, to: {n}, by: 1]:")
            out.append(f"        for $b: u64 [from: 1, to: {n}, by: 1]:")
            out.append(f"            let $x := {prefix}::str.slice.{unit}(lend $s{k}, {ab})-")
            out += otherwise(12, f'std::print.stored("{tag} sl{u} {k} ", $a, " ", $b, " out_of_range", newln); continue;')
            out.append(f'            std::print.stored("{tag} sl{u} {k} ", $a, " ", $b, " [", $x, "]", newln);')
            out.append("        ;for")
            out.append("    ;for")
out.append(";MAIN")
print("\n".join(out))
