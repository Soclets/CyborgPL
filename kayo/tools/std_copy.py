#!/usr/bin/env python3
"""A redesign-three copy of std/ for kayo's tests, until std claude converts std/ itself (the redesign-three
conversion, 2026-10-29): std/ copied to .kayo-checks/std3, the oracle's converters run over it in order
(`oracle --redesign`, then `oracle/tools/redesign_three.py`), then the fixes they cannot make by themselves.
Point KAYO_STD at it.

    python3 kayo/tools/std_copy.py

`plain_verbs.py` takes a verb off a plain value by running the oracle on a program, and std is not one; here
kayo's own refusal (`nothing_to_give`, building examples/hello.cy against the copy) says where, a round each.
"""

import os
import re
import shutil
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OUT = REPO / ".kayo-checks/std3"
KAYO = REPO / "kayo/target/release/kayo"

shutil.rmtree(OUT, ignore_errors=True)
shutil.copytree(REPO / "std", OUT, ignore=shutil.ignore_patterns("tools"))
subprocess.run([str(REPO / "oracle/target/release/oracle"), "--redesign", str(OUT)], check=True, capture_output=True)
subprocess.run(["python3", str(REPO / "oracle/tools/redesign_three.py"), str(OUT)], check=True, capture_output=True)

# The converters' slips, by hand: `lt`, Lithuanian, is a case of `casing`, not the comparison.
kit = OUT / "kit.cy"
kit.write_text(kit.read_text().replace("    tr; az; <; root;", "    tr; az; lt; root;"))

# `text_of` stops with a print and an exit when std's own cut is not UTF-8 (a bug in std), which makes it a
# `proc`, and `split` and `join`, which call it, could not be `fn`s. Until std claude decides how std stops on its
# own bug, the copy gives the empty text there instead.
s = (OUT / "str.cy").read_text()
start = s.index("proc text_of [let $bytes: ls[u8]] -> str.utf8:")
end = s.index(";proc text_of", start) + len(";proc text_of")
s = s[:start] + "fn text_of [let $bytes: ls[u8]] -> str.utf8:\n    ret to.str.utf8(lend $bytes) ?? str.empty;\n;fn text_of" + s[end:]
(OUT / "str.cy").write_text(s)

# What std offers is the kind the oracle's built-in std has: rounding is a `fn`. (std's own functions read from
# this copy are not checked for effects, its stops on its own bugs printing and exiting.)
r = (OUT / "round.cy").read_text()
(OUT / "round.cy").write_text(r.replace("pub coll.proc ", "pub coll.fn ").replace(";coll.proc ", ";coll.fn "))

# A verb on a plain value, a round each.
env = dict(os.environ, KAYO_STD=str(OUT))
AT = re.compile(r"error\[(nothing_to_give|owned_value_bare)\] at (.*):(\d+):(\d+)")
for _ in range(500):
    r = subprocess.run([str(KAYO), "build", str(REPO / "examples/hello.cy"), "-o", str(OUT / "hello")], env=env, capture_output=True, text=True)
    m = AT.search(r.stderr)
    if not m or not m.group(2).startswith(str(OUT)):
        break
    kind, path, line, col = m.group(1), Path(m.group(2)), int(m.group(3)), int(m.group(4))
    lines = path.read_text().split("\n")
    text = lines[line - 1]
    at = col - 1
    if kind == "owned_value_bare":
        # A conversion lends what it only reads (owner, 2026-10-28).
        if "only looks at" not in r.stderr and "(lend $" not in r.stderr:
            raise SystemExit(f"{path}:{line}:{col}: {r.stderr}")
        lines[line - 1] = text[:at] + "lend " + text[at:]
    else:
        verb = re.match(r"(copy|give|lend\.mut|lend) ", text[at:])
        if not verb:
            raise SystemExit(f"{path}:{line}:{col}: no verb to take off: {text}")
        lines[line - 1] = text[:at] + text[at + len(verb.group(0)):]
    path.write_text("\n".join(lines))
(OUT / "hello").unlink(missing_ok=True)
print(OUT)
