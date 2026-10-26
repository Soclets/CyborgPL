"""Marks the limb words in kayo's own messages (owner, 2026-10-26: every piece a limb adds is marked `%`), so the code a
message shows or a fix writes reads as a program must: only inside `…` in a Rust string literal, never in a comment,
never after a kit's `::`, never twice.

    python3 -I kayo/tools/mark_messages.py kayo/compiler/src     # prints each file it changes
"""
import os
import re
import sys

# The oracle's table (design/cores.md, "The limb mark"), the first word of each limb's path.
RULES = [
    (r"(?<![\w.%:$'])(extern|layout)\.(?=[\w{])", r"%\1."),
    (r"(?<![\w.%:$'])C\.(?=[\w{])", r"%C."),
    (r"\bto\.(d32|d64|d128|i128|u128|f128|brain16)\b", r"to.%\1"),
    (r"(?<![\w.%:$'])(d32|d64|d128|i128|u128|f128|brain16)\b", r"%\1"),
    (r"(?<![\w.%:$'])not_std\.(?=[\w{])", r"%not_std."),
    (r"(?<![\w.%:$'])(concur|channel|whichever)\b", r"%\1"),
    (r"\bfor para\b", r"for %para"),
    (r"\bincl\.C\b", r"incl.%C"),
    (r"\binline\.(C|MSL)\b", r"inline.%\1"),
    (r"\bchar\.(point|grapheme)\b", r"char.%\1"),
    (r"\bstr\.(utf16|utf32)\b", r"str.%\1"),
    (r"(\d) (graphemes|points|grapheme|point)\b", r"\1 %\2"),
]


def mark_code(code):
    for pat, rep in RULES:
        code = re.sub(pat, rep, code)
    return code


def mark_string(lit):
    # Backticked code inside one string literal's text; a last one left open
    # (a phrase cut short) is code too.
    parts = lit.split("`")
    return "`".join(mark_code(p) if n % 2 == 1 else p for n, p in enumerate(parts))


def mark_line(line):
    if line.lstrip().startswith("//"):
        return line
    out, i = [], 0
    for m in re.finditer(r'"(?:[^"\\]|\\.)*"', line):
        out.append(line[i:m.start()])
        out.append(mark_string(m.group(0)))
        i = m.end()
    out.append(line[i:])
    return "".join(out)


for root in sys.argv[1:]:
    for dirpath, _, names in os.walk(root):
        for n in names:
            if not n.endswith(".rs"):
                continue
            p = os.path.join(dirpath, n)
            old = open(p, encoding="utf-8").read()
            new = "\n".join(mark_line(l) for l in old.split("\n"))
            if new != old:
                open(p, "w", encoding="utf-8").write(new)
                print(p)
