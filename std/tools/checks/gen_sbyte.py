# gen_sbyte.py real|python: str.slice.byte over every position pair, each slice printed as its bytes.
# real: std/ through kayo. python: what it should give, by Python's own bytes.
import sys

mode = sys.argv[1]
texts = ["", "a", "abc", "h\xe9llo", "\xe9\U0001f34e\xe9", "a\U0001f468\u200d\U0001f469\u200d\U0001f467b",
         "\u0e44\u0e17\u0e22"]


def q(s):
    return "str.empty" if s == "" else '"' + s + '"'


if mode == "python":
    for k, t in enumerate(texts, 1):
        b = t.encode()
        n = len(b) + 2
        for x in range(1, n + 1):
            for y in range(1, n + 1):
                if y < x or y > len(b):
                    print(f"slb {k} {x} {y} out_of_range")
                else:
                    print(f"slb {k} {x} {y} [" + "".join(f" {v}" for v in b[x - 1:y]) + " ]")
    sys.exit(0)

out = ["MAIN {"]
for k, t in enumerate(texts, 1):
    out.append(f"    let $s{k} := {q(t)};")
    n = len(t.encode()) + 2
    out.append(f"    for $a: u64 [from: 1, to: {n}, by: 1] {{")
    out.append(f"        for $b: u64 [from: 1, to: {n}, by: 1] {{")
    out.append(f'            let $x := std::str.slice.byte(lend $s{k}, from: pos $a, to: pos $b) otherwise {{ out_of_range {{ std::print.stored("slb {k} ", $a, " ", $b, " out_of_range", newln); continue; }} }};')
    out.append(f'            std::print.stored("slb {k} ", $a, " ", $b, " [");')
    out.append('            for $v in lend $x { std::print.stored(" ", $v); }')
    out.append('            std::print.stored(" ]", newln);')
    out.append("        }")
    out.append("    }")
out.append("}")
print("\n".join(out))
