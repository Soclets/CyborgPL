#!/usr/bin/env python3
"""A written-out default goes (owner, 2026-10-29: "refuse"): `std::print.shortest(…)` → `std::print(…)`,
`std::print.shortest.stderr` → `std::print.stderr`, `std::print.arrows.shortest` → `std::print.arrows`, `for temp $i` →
`for $i`, a parameter's `let`. Code only, never comments or strings; in a `.md` file, only inside ``` blocks.

  python3 oracle/tools/written_defaults.py FILE...
"""
import importlib.util, os, re, sys

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("redesign_three", os.path.join(here, "redesign_three.py"))
r3 = importlib.util.module_from_spec(spec); spec.loader.exec_module(r3)
spec = importlib.util.spec_from_file_location("power_std", os.path.join(here, "power_std.py"))
ps = importlib.util.module_from_spec(spec)
sys.argv, argv = [sys.argv[0]], sys.argv[1:]
spec.loader.exec_module(ps)


def defaults(code):
    code = re.sub(r'std::print\.shortest(?=\.std(?:err|out)\b|\(|;|\s+as\b)', 'std::print', code)
    code = re.sub(r'std::print\.arrows\.shortest\b', 'std::print.arrows', code)
    code = re.sub(r'\bfor\s+((?:%para\s+)?)temp\s+', r'for \1', code)
    return r3.PARAM_LET.sub(r'\1', code)


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
        new += line[last:a] + defaults(line[a:b]); last = b
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
