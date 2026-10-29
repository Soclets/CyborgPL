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

# A verb on a plain value, a round each.
env = dict(os.environ, KAYO_STD=str(OUT))
AT = re.compile(r"error\[nothing_to_give\] at (.*):(\d+):(\d+)")
for _ in range(500):
    r = subprocess.run([str(KAYO), "build", str(REPO / "examples/hello.cy"), "-o", str(OUT / "hello")], env=env, capture_output=True, text=True)
    m = AT.search(r.stderr)
    if not m:
        break
    path, line, col = Path(m.group(1)), int(m.group(2)), int(m.group(3))
    lines = path.read_text().split("\n")
    text = lines[line - 1]
    at = col - 1
    verb = re.match(r"(copy|give|lend\.mut|lend) ", text[at:])
    if not verb:
        raise SystemExit(f"{path}:{line}:{col}: no verb to take off: {text}")
    lines[line - 1] = text[:at] + text[at + len(verb.group(0)):]
    path.write_text("\n".join(lines))
(OUT / "hello").unlink(missing_ok=True)
print(OUT)
