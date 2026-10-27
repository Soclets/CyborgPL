#!/usr/bin/env python3
"""The redesign's sweep of the design notes (2026-10-27): each fenced code block in design/*.md is rewritten by
`oracle --redesign-show`, alone; prose, quotes and history are never touched (owner, 2026-10-27: "Better becareful with
quoted design shit"). A block that does not convert (a fragment with `…`, an old spelling shown on purpose) is left as
it is and listed. `python3 oracle/tools/redesign_notes.py [--write]`: without `--write`, prints what would change."""
import os, re, subprocess, sys, tempfile
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import redesign_fields

repo = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
oracle = os.path.join(repo, "oracle", "target", "debug", "oracle")
notes = os.path.join(repo, "design")
# history, kept in the spellings of its day
SKIP = {"revision-2026-10-07.md", "lilith-session.md", "synthia-session.md", "synthia-manifest.md", "rrx2-lang-session.md", "rrx2-tv-session.md"}
write = "--write" in sys.argv
fence = re.compile(r'^(```[a-z]*)\n(.*?)^```', re.S | re.M)
def fields_and_counts(code):
    """A record's fields one per line, and each `#generated_N_lines_below` counting the lines under it now"""
    with tempfile.NamedTemporaryFile("w", suffix=".cy", delete=False) as f:
        f.write(code)
    import io, contextlib
    with contextlib.redirect_stdout(io.StringIO()):
        redesign_fields.fix(f.name)
    code = open(f.name).read()
    os.unlink(f.name)
    lines = code.split("\n")
    for i, l in enumerate(lines):
        m = re.match(r"^(\s*)#generated_\d+_lines_below\s*$", l)
        if m:
            ind = len(m.group(1))
            n = 0
            for x in lines[i + 1:]:
                if not x.strip() or len(x) - len(x.lstrip()) < ind or (len(x) - len(x.lstrip()) == ind and x.strip().startswith("end of ") and False):
                    break
                n += 1
            # the lines up to the collection's own `end of` at the marker's depth less one
            lines[i] = f"{m.group(1)}#generated_{n}_lines_below"
    return "\n".join(lines)

changed, left = 0, []
for name in sorted(os.listdir(notes)):
    if not name.endswith(".md") or name in SKIP:
        continue
    path = os.path.join(notes, name)
    text = open(path).read()
    out, pos = [], 0
    for m in fence.finditer(text):
        lang, code = m.group(1), m.group(2)
        out.append(text[pos:m.start()])
        new = code
        # only CyborgPL: a block with code's marks, not a commit trailer, a shell line or a manifest
        # the 2026-09 spellings, shown as history (`var.f16 'x'`, `func.i32`, `loop.temp.range`), stay
        historic = re.search(r"\b(var|func|loop\.temp)\.", code) is not None
        looks_like_code = not historic and any(k in code for k in ("{", ";", "MAIN", "fn ", "let ")) and not re.search(r"Co-Authored-By|^\s*(\$ |cargo |kayo |oracle |python3 |git )|^\s*\w+ = ", code, re.M)
        if lang in ("```", "```cy", "```cyborgpl") and looks_like_code:
            with tempfile.NamedTemporaryFile("w", suffix=".cy", delete=False) as f:
                f.write(code)
            r = subprocess.run([oracle, "--redesign-show", f.name], capture_output=True, text=True)
            os.unlink(f.name)
            if r.returncode == 0:
                new = fields_and_counts(r.stdout)
            else:
                left.append(f"{name}: line {text[:m.start()].count(chr(10)) + 1}: {r.stderr.strip().split(': ', 1)[-1][:120]}")
        if new != code:
            changed += 1
            if not write:
                print(f"--- {name} at line {text[:m.start()].count(chr(10)) + 1}\n{code}+++\n{new}")
        out.append(f"{lang}\n{new}```")
        pos = m.end()
    out.append(text[pos:])
    if write:
        open(path, "w").write("".join(out))
print(f"{changed} blocks {'rewritten' if write else 'would change'}; left as they are:", file=sys.stderr)
for l in left:
    print("  " + l, file=sys.stderr)
