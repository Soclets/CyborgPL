#!/usr/bin/env python3
"""A generic's placeholders move into their own `[ ]` (owner, 2026-10-29): `fn total <T: Addable>` becomes
`fn total [<T>: Addable]`. Code only, never comments or strings; in a `.md` file, only inside ``` blocks.

  python3 oracle/tools/generic_brackets.py FILE...
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
    new, last = "", 0
    for a, b in ps.code_spans(line):
        new += line[last:a] + r3.generic_lists(line[a:b]); last = b
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
