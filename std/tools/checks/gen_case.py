# gen_case.py pair|real UNITS: str.upper/lower.<unit> over a set of texts, the units one of ascii, simple, full,
# full.root, full.tr, full.az, full.lt (e.g. ascii,simple,full,full.root,full.tr,full.az,full.lt).
# pair: "A …" the oracle's built-in std::str.X, "B …" stdtest's (std/ copied); real: "A …" std/ through kayo.
import sys

mode, units = sys.argv[1], sys.argv[2].split(",")
texts = ["", "abc", "ABC", "Hello, World!", "caf\xe9", "CAF\xc9", "stra\xdfe", "\u01c5", "\ufb00", "\u0149",
         "\u0130stanbul", "ISTANBUL", "\u039f\u0394\u03a5\u03a3\u03a3\u0395\u03a5\u03a3", "\u03a3",
         "a\u03a3 b", "\u0e44\u0e17\u0e22", "\U0001f34ea\U0001f34e", "\u01f0", "\u1fb3", "\u03a9",
         "x\u0301y", "\u01c4\u01c5\u01c6", "\u0390", "i\u0307", "I\u0307", "\u1f00\u0390"]
# context: final sigma (ʰ and U+0345 are cased and case-ignorable both), the Turkish and Lithuanian dots
texts += ["\u0391\u03a3", "\u0391\u03a3.", "\u0391\u03a3'\u0391", "\u0391'\u03a3", "\u03a3\u0391",
          "\u0391\u03a3\u0391", "\u03a3\u02b0", "\u02b0\u03a3", "\u0391\u03a3\u02b0", "\u0391\u0345\u03a3",
          "\u0391\u03a3\u0301", "\u0391\u03a3\u0301\u0392", "I\u0307", "I\u0316\u0307", "I\u0300\u0307",
          "IJ\u0301", "J\u0301", "\u012e\u0303", "\xcc\xcd\u0128", "I\u0328\u0301", "i\u0307",
          "i\u0307\u0300", "j\u0323\u0307", "\u012f\u0307", "a\u0307", "i\u0131", "istanbul",
          "D\u0130YARBAKIR"]


def q(s):
    return "str.empty" if s == "" else '"' + s + '"'


out = ["MAIN:"]
sides = [("A", "std::", True)] + ([("B", "stdtest::", False)] if mode == "pair" else [])
for k, t in enumerate(texts, 1):
    out.append(f"    let $s{k} := {q(t)};")
    for tag, pre, builtin in sides:
        for f in ["upper", "lower"]:
            for u in units:
                if u.startswith("full."):
                    # `std::` calls take `lang:` (spellings.cyr's calls); the oracle reads stdtest's in place
                    word = "lang: " if builtin else ""
                    call = f"{pre}str.{f}.full(lend $s{k}, {word}{pre}casing.{u[5:]})"
                else:
                    call = f"{pre}str.{f}.{u}(lend $s{k})"
                out.append(f'    std::print.stored("{tag} {f}.{u} {k} [", {call}, "]", newln);')
out.append(";MAIN")
print("\n".join(out))
