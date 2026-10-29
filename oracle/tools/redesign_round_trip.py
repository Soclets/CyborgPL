#!/usr/bin/env python3
"""The redesign's round trip (2026-10-27): copy examples/ (or another folder) to a scratch folder, rewrite the copy with
`oracle --redesign` and then `redesign_three.py`, run every program there with the oracle, and compare with its `.out` / `.exit`. Run from the repo:
`python3 oracle/tools/redesign_round_trip.py [folder] [scratch]`."""
import os, shutil, subprocess, sys, tempfile

repo = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
# the newer of the two builds, release or debug, whichever exists
oracle = max((p for p in (os.path.join(repo, "oracle", "target", k, "oracle") for k in ("release", "debug")) if os.path.exists(p)),
             key=os.path.getmtime, default=os.path.join(repo, "oracle", "target", "debug", "oracle"))
src = sys.argv[1] if len(sys.argv) > 1 else os.path.join(repo, "examples")
work = sys.argv[2] if len(sys.argv) > 2 else os.path.join(tempfile.gettempdir(), "cyborgpl-round-trip")
shutil.rmtree(work, ignore_errors=True)
shutil.copytree(src, work)
conv = subprocess.run([oracle, "--redesign", work], capture_output=True, text=True)
for line in conv.stderr.splitlines():
    print("CONVERT", line)
# then redesign two to three (2026-10-28), and a plain union's verbs taken off where the oracle names them
tools = os.path.join(repo, "oracle", "tools")
subprocess.run([sys.executable, os.path.join(tools, "redesign_three.py"), work], capture_output=True, text=True)
for name in sorted(os.listdir(work)):
    p = os.path.join(work, name)
    if name.endswith(".cy") or (os.path.isdir(p) and os.path.isfile(os.path.join(p, "kit.cy"))):
        subprocess.run([sys.executable, os.path.join(tools, "plain_verbs.py"), p], capture_output=True, text=True)
passed, failed = 0, 0
names = sorted(os.listdir(work))
for name in names:
    path = os.path.join(work, name)
    if name.endswith(".cy"):
        base = path[:-3]
    elif os.path.isdir(path) and os.path.isfile(os.path.join(path, "kit.cy")):
        base = path
    else:
        continue
    if os.path.exists(base + ".oracle") or not os.path.exists(base + ".out"):
        continue
    args = []
    if os.path.exists(base + ".args"):
        args = ["--"] + open(base + ".args").read().splitlines()
    try:
        r = subprocess.run([oracle, path] + args, capture_output=True, stdin=subprocess.DEVNULL, timeout=300, cwd=work)
    except subprocess.TimeoutExpired:
        failed += 1
        print(f"FAIL {name}: timed out")
        continue
    want = open(base + ".out", "rb").read()
    code = 0
    if os.path.exists(base + ".exit"):
        code = int(open(base + ".exit").read().strip().removeprefix("exit:"))
    if r.stdout == want and r.returncode == code:
        passed += 1
    else:
        failed += 1
        print(f"FAIL {name} ({r.returncode}): {r.stderr.decode(errors='replace')[:240]!r}")
print(f"pass {passed} fail {failed}")
