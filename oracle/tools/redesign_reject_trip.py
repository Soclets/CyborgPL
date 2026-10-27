#!/usr/bin/env python3
"""The redesign's reject round trip (2026-10-27): copy oracle/tests/reject to a scratch folder, rewrite the copy with
`oracle --redesign`, and check that each test still fails with a message holding its `.err`. Prints each one that does
not, with what it says now. `python3 oracle/tools/redesign_reject_trip.py [scratch]`"""
import os, shutil, subprocess, sys, tempfile

repo = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
oracle = os.path.join(repo, "oracle", "target", "debug", "oracle")
src = os.path.join(repo, "oracle", "tests", "reject")
work = sys.argv[1] if len(sys.argv) > 1 else os.path.join(tempfile.gettempdir(), "cyborgpl-reject-trip")
shutil.rmtree(work, ignore_errors=True)
shutil.copytree(src, work)
conv = subprocess.run([oracle, "--redesign", work], capture_output=True, text=True)
unconverted = set()
for line in conv.stderr.splitlines():
    if ": line " in line or "overlap" in line or "block open" in line:
        unconverted.add(os.path.basename(line.split(":")[0]))
passed, failed = 0, 0
for name in sorted(os.listdir(work)):
    path = os.path.join(work, name)
    if name.endswith(".cy"):
        err = path[:-3] + ".err"
    elif os.path.isdir(path) and os.path.isfile(path + ".err"):
        err = path + ".err"
    else:
        continue
    if not os.path.isfile(err):
        continue
    want = open(err).read().strip()
    r = subprocess.run([oracle, path], capture_output=True, stdin=subprocess.DEVNULL, timeout=120, cwd=work)
    got = r.stderr.decode(errors="replace")
    if r.returncode != 0 and want in got:
        passed += 1
    else:
        failed += 1
        tag = " (not rewritten)" if name in unconverted else ""
        print(f"FAIL {name}{tag}\n  want: {want[:160]}\n  got:  {got.strip()[:220]}")
print(f"pass {passed} fail {failed}")
