# gen_text.py pair|real: str.contains / starts_with / ends_with / split / join / replace over texts and parts.
# pair: each line "label <the oracle's built-in std> ||| <stdtest's, this std/ copied>", run in the oracle.
# real: "label <std/'s>", built by kayo against std/.
import sys

mode = sys.argv[1]
texts = ["", "a", "abc", "red,green,blue", ",a,,b,", "aaa", "aaaa", "h\xe9llo w\xf6rld",
         "\xe9\U0001f34e\xe9", "\U0001f34e\U0001f34e\U0001f34e", "a, b, c", "xyx"]
parts = ["a", ",", "aa", "é", "🍎", "🍎🍎", "xyx", "x", "lo w", ", ", "zzzz"]


def q(s):
    return "str.empty" if s == "" else '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


out = ["use std::list as list;", "MAIN {"]


def show(label, a, b):
    if mode == "pair":
        out.append(f'    std::print.stored("{label} ", {a}, " ||| ", {b}, newln);')
    else:
        out.append(f'    std::print.stored("{label} ", {a}, newln);')


def call(f, words, plain):
    """std's call with its argument words, and stdtest's with the same arguments in place."""
    return f"std::str.{f}({', '.join(words)})", f"stdtest::str.{f}({', '.join(plain)})"


k = 0
for t in texts:
    for p in parts:
        k += 1
        out.append(f"    let $s{k} := {q(t)};")
        out.append(f"    let $p{k} := {q(p)};")
        for f in ["contains", "starts_with", "ends_with"]:
            show(f"{f} {k}", *call(f, [f"lend $s{k}", f"part: lend $p{k}"], [f"lend $s{k}", f"lend $p{k}"]))
        a, b = call("split", [f"lend $s{k}", f"at: lend $p{k}"], [f"lend $s{k}", f"lend $p{k}"])
        out.append(f"    let $x{k} := {a};")
        if mode == "pair":
            out.append(f"    let $y{k} := {b};")
        show(f"split {k}", f'list.length(lend $x{k}), ":", std::str.join(lend $x{k}, between: "|")',
             f'list.length(lend $y{k}), ":", std::str.join(lend $y{k}, between: "|")')
        for to, lab in [('"<>"', "replace"), ("str.empty", "replace0")]:
            a, b = call("replace", [f"lend $s{k}", f"from: lend $p{k}", f"to: {to}"], [f"lend $s{k}", f"lend $p{k}", to])
            show(f"{lab} {k}", f'"[", {a}, "]"', f'"[", {b}, "]"')
lists = [[], [""], ["a"], ["a", "b"], ["", ""], ["é", "🍎", ""], ["red", "green", "blue"]]
for i, l in enumerate(lists):
    lit = "[" + ", ".join(q(x) for x in l) + ("," if len(l) == 1 else "") + "]"
    out.append(f"    let $l{i}: list[str.utf8] = {lit};")
    for bt in ["", ",", " and ", "🍎"]:
        k += 1
        out.append(f"    let $b{k} := {q(bt)};")
        a, b = call("join", [f"lend $l{i}", f"between: lend $b{k}"], [f"lend $l{i}", f"lend $b{k}"])
        show(f"join {k}", f'"[", {a}, "]"', f'"[", {b}, "]"')
out.append("}")
print("\n".join(out))
