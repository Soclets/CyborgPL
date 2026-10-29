#!/usr/bin/env python3
"""A parameter's `let` goes (owner, 2026-10-29: "(c)", a name inside brackets is fixed unless it says `var`):
`fn add [let $a: i32]` becomes `fn add [$a: i32]`. Code only, never comments or strings; in a `.md` file, only inside
``` blocks.

  python3 oracle/tools/param_let.py FILE...
"""
import importlib.util, os, sys

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("redesign_three", os.path.join(here, "redesign_three.py"))
r3 = importlib.util.module_from_spec(spec); spec.loader.exec_module(r3)
spec = importlib.util.spec_from_file_location("power_std", os.path.join(here, "power_std.py"))
ps = importlib.util.module_from_spec(spec)
sys.argv, argv = [sys.argv[0]], sys.argv[1:]
spec.loader.exec_module(ps)


def code_line(line):
    # a parameter's `let` at the start of a continued line is decided on the whole line, its strings and quoted names
    # blanked, so a name like `$'x"y'` cannot hide a declaration's `=` (compiler claude, 2026-10-29)
    spans = ps.code_spans(line)
    masked, last = "", 0
    for a, b in spans:
        masked += "x" * (a - last) + line[a:b]; last = b
    masked += "x" * (len(line) - last)
    m = r3.PARAM_LINE_LET.match(masked)
    if m: line = line[:m.end(1)] + line[m.end():]
    new, last = "", 0
    for a, b in ps.code_spans(line):
        new += line[last:a] + r3.PARAM_LET.sub(r'\1', line[a:b]); last = b
    return new + line[last:]


for path in argv:
    lines = open(path, encoding="utf-8").read().split("\n")
    out, in_block = [], not path.endswith(".md")
    for line in lines:
        if path.endswith(".md") and line.lstrip().startswith("```"):
            in_block = not in_block; out.append(line); continue
        out.append(code_line(line) if in_block else line)
    if out != lines:
        open(path, "w", encoding="utf-8").write("\n".join(out)); print(path)
