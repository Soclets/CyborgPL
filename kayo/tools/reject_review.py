#!/usr/bin/env python3
"""Every reject test as conformance.rs's `reject_programs_are_refused_in_ccas_own_words` judges it, quicker to
read: its phrase line, the `.err` that line was reviewed against, kayo's message (with its sentence shown beside
a written fix, `KAYO_SHOW_ABOUT`) and kayo's error name against errors.cyr's. Prints only what fails.

  python3 kayo/tools/reject_review.py [NAME ...]

Runs kayo/target/release/kayo three at a time, KAYO_STD as set.
"""

import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import phrases  # noqa: E402

REPO = Path(__file__).resolve().parents[2]
KAYO = REPO / "kayo/target/release/kayo"
REJECT = REPO / "oracle/tests/reject"


def error_names():
    text = (REPO / "errors.cyr").read_text()
    out = {}
    for m in re.finditer(r'name = "([^"]+)".*?tests = \[([^\]]*)\]', text):
        for t in re.findall(r'"([^"]+)"', m.group(2)):
            out[t] = m.group(1)
    return out


def exceptions():
    path = REPO / "kayo/compiler/tests/name_exceptions.txt"
    out = {}
    for line in path.read_text().splitlines():
        if line.strip() and not line.startswith("#") and " :: " in line:
            test, rest = line.split(" :: ", 1)
            out[test.strip()] = rest.split(" — ")[0].strip()
    return out


def judge(name, entries, names, excepted):
    path = REJECT / name if (REJECT / name).is_dir() else REJECT / f"{name}.cy"
    err_path = REJECT / f"{name}.err"
    oracle = err_path.read_text().strip() if err_path.is_file() else ""
    env = dict(os.environ, KAYO_SHOW_ABOUT="1")
    r = subprocess.run([str(KAYO), "build", str(path), "-o", "/tmp/kayo-reject-review"], capture_output=True, text=True, env=env, stdin=subprocess.DEVNULL)
    msg = r.stderr
    lines = [l for l in msg.splitlines() if l.startswith(("error", "check", "about:", "fix:", "code:"))][:4]
    shown = "\n    ".join(lines)
    if name not in entries:
        return f"{name}: NO LINE\n    oracle: {oracle}\n    {shown}"
    _, mark, phrase = entries[name]
    problems = []
    if err_path.is_file() and mark != f"[fnv:{phrases.fnv(err_path)}]":
        problems.append("MOVED")
    if phrase.startswith("!"):
        if r.returncode == 0 and not (phrase.startswith("!pending") or "by-design" in phrase):
            problems.append("ACCEPTED")
    elif r.returncode == 0:
        problems.append("ACCEPTED")
    elif phrase not in msg:
        problems.append("PHRASE")
    said = re.search(r"^(?:error|check)\[([^\]]+)\]", msg, re.M)
    want = names.get(name)
    if r.returncode != 0 and not phrase.startswith("!"):
        if want is None:
            problems.append("NO NAME")
        elif said and said.group(1) != want and excepted.get(name) != said.group(1):
            problems.append(f"NAME {said.group(1)} (errors.cyr: {want})")
    if not problems:
        return None
    return f"{name}: {' '.join(problems)}\n    phrase: {phrase}\n    oracle: {oracle}\n    {shown}"


def main():
    _, entries = phrases.read()
    names = error_names()
    excepted = exceptions()
    every = sorted(phrases.tests())
    wanted = sys.argv[1:] or every
    with ThreadPoolExecutor(3) as pool:
        results = list(pool.map(lambda n: judge(n, entries, names, excepted), wanted))
    bad = [r for r in results if r]
    for r in bad:
        print(r)
    print(f"\n{len(wanted) - len(bad)} hold, {len(bad)} do not, of {len(wanted)}")


if __name__ == "__main__":
    main()
