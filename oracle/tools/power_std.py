#!/usr/bin/env python3
"""`^` and a bare `pow` become `std::pow` (owner, 2026-10-28): code only, never comments or strings.

  python3 oracle/tools/power_std.py FILE...
"""
import re, sys

def code_spans(line):
    """(start, end) of the code in a line: outside strings, chars and the comment."""
    spans, i, start, n = [], 0, 0, len(line)
    while i < n:
        c = line[i]
        if c == '"':
            spans.append((start, i)); j = i + 1
            while j < n and line[j] != '"': j += 2 if line[j] == "\\" else 1
            i = start = j + 1; continue
        if c == "'" and i > 0 and line[i - 1] == "$":
            # a quoted name, `$'x"y'`: its characters are the name's, never a string's or a comment's
            j = line.find("'", i + 1); i = (j + 1) if j > 0 else n; continue
        if c == "`":
            spans.append((start, i)); j = line.find("`", i + 1); i = start = (j + 1 if j > 0 else n); continue
        if c == "#":
            spans.append((start, i)); return spans
        i += 1
    spans.append((start, n)); return spans

def convert(code):
    code = re.sub(r"(?<=\s)\^(?=\s)", "std::pow", code)
    code = re.sub(r"(?<![\w:.$'])pow(?=\.(wrap|ieee)\b)", "std::pow", code)
    code = re.sub(r"(?<!\bop )(?<![\w:.$'])pow(?![\w.])", "std::pow", code)
    return code

for path in sys.argv[1:]:
    lines = open(path, encoding="utf-8").read().split("\n")
    out = []
    for line in lines:
        new, last = "", 0
        for a, b in code_spans(line):
            new += line[last:a] + convert(line[a:b]); last = b
        out.append(new + line[last:])
    if out != lines:
        open(path, "w", encoding="utf-8").write("\n".join(out)); print(path)
