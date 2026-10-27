#!/usr/bin/env python3
"""Redesign two's round trip (2026-10-27): copy examples/ (or another folder) to a scratch folder, shorten the copy
from redesign one's words to two's with `oracle --redesign-shorten`, run every program there, and compare with its
`.out` / `.exit`. `python3 oracle/tools/redesign_two_trip.py [folder] [scratch]`"""
import os, shutil, subprocess, sys, tempfile

repo = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
oracle = os.path.join(repo, "oracle", "target", "debug", "oracle")
src = sys.argv[1] if len(sys.argv) > 1 else os.path.join(repo, "examples")
work = sys.argv[2] if len(sys.argv) > 2 else os.path.join(tempfile.gettempdir(), "cyborgpl-two-trip")
shutil.rmtree(work, ignore_errors=True)
shutil.copytree(src, work)
conv = subprocess.run([oracle, "--redesign-shorten", work], capture_output=True, text=True)
for line in conv.stderr.splitlines():
    print("SHORTEN", line)
passed, failed = 0, 0
for name in sorted(os.listdir(work)):
    path = os.path.join(work, name)
    if name.endswith(".cy"): base = path[:-3]
    elif os.path.isdir(path) and os.path.isfile(os.path.join(path, "kit.cy")): base = path
    else: continue
    if os.path.exists(base + ".oracle") or not os.path.exists(base + ".out"): continue
    args = ["--"] + open(base + ".args").read().splitlines() if os.path.exists(base + ".args") else []
    r = subprocess.run([oracle, path] + args, capture_output=True, stdin=subprocess.DEVNULL, timeout=300, cwd=work)
    code = int(open(base + ".exit").read().strip().removeprefix("exit:")) if os.path.exists(base + ".exit") else 0
    if r.stdout == open(base + ".out", "rb").read() and r.returncode == code: passed += 1
    else:
        failed += 1
        print(f"FAIL {name} ({r.returncode}): {r.stderr.decode(errors='replace')[:240]!r}")
print(f"pass {passed} fail {failed}")
