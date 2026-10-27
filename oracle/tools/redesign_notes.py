#!/usr/bin/env python3
"""The redesign's sweep of the design notes (2026-10-27): each fenced code block in design/*.md is rewritten by
`oracle --redesign-show`, alone; prose, quotes and history are never touched (owner, 2026-10-27: "Better becareful with
quoted design shit"). A block that does not convert (a fragment with `…`, an old spelling shown on purpose) is left as
it is and listed. `python3 oracle/tools/redesign_notes.py [--write]`: without `--write`, prints what would change."""
import os, re, subprocess, sys, tempfile

repo = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
oracle = os.path.join(repo, "oracle", "target", "debug", "oracle")
notes = os.path.join(repo, "design")
# history, kept in the spellings of its day
SKIP = {"revision-2026-10-07.md"}
write = "--write" in sys.argv
fence = re.compile(r'^(```[a-z]*)\n(.*?)^```', re.S | re.M)
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
        if lang in ("```", "```cy", "```cyborgpl"):
            with tempfile.NamedTemporaryFile("w", suffix=".cy", delete=False) as f:
                f.write(code)
            r = subprocess.run([oracle, "--redesign-show", f.name], capture_output=True, text=True)
            os.unlink(f.name)
            if r.returncode == 0:
                new = r.stdout
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
