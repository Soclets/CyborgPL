#!/usr/bin/env python3
"""Where kayo stands on the oracle's tests, quickly (the redesign-three conversion, 2026-10-29).

  python3 kayo/tools/examples_status.py              every example: built, run, its output against the oracle's
  python3 kayo/tools/examples_status.py --reject     every reject test: refused with its phrase
  python3 kayo/tools/examples_status.py NAME ...     only these

Runs `kayo/target/release/kayo` (build it first), three at a time (the session's cores), with `KAYO_STD`
as set in the environment. Prints each failure's first lines and the counts; nothing is written.
"""

import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
KAYO = REPO / "kayo/target/release/kayo"
EXAMPLES = REPO / "examples"
REJECT = REPO / "oracle/tests/reject"
PHRASES = REPO / "kayo/compiler/tests/reject_phrases.txt"
SCRATCH = Path(os.environ.get("TMPDIR", "/tmp")) / "kayo-examples-status"


def targets(folder: Path, names):
    out = []
    for p in sorted(folder.iterdir()):
        if p.suffix == ".cy" or (p.is_dir() and (p / "kit.cy").is_file()):
            name = p.stem if p.suffix == ".cy" else p.name
            if not names or name in names:
                out.append((name, p))
    return out


def first_lines(text: str, n: int = 4) -> str:
    return "\n".join("    " + l for l in text.strip().splitlines()[:n])


def run_example(item):
    name, path = item
    want_p = EXAMPLES / f"{name}.out"
    if not want_p.is_file():
        return name, None, "no .out"
    want = want_p.read_bytes()
    code = 0
    exit_p = EXAMPLES / f"{name}.exit"
    if exit_p.is_file():
        code = int(exit_p.read_text().strip().rsplit(":", 1)[-1].strip())
    args_p = EXAMPLES / f"{name}.args"
    words = [l for l in args_p.read_text().splitlines() if l] if args_p.is_file() else []
    if path.is_dir() and (path / "CyJet.toml").is_file() and "aarch64-none" in (path / "CyJet.toml").read_text(errors="ignore"):
        return name, None, "bare (QEMU), not run here"
    cmd = [str(KAYO), "run", str(path)] + (["--"] + words if words else [])
    try:
        r = subprocess.run(cmd, stdin=subprocess.DEVNULL, capture_output=True, timeout=120)
    except subprocess.TimeoutExpired:
        return name, False, "timed out"
    if r.stdout == want and r.returncode == code:
        return name, True, ""
    err = r.stderr.decode(errors="replace")
    if any(l.startswith(("error[", "error:")) for l in err.splitlines()) and not r.stdout:
        return name, False, first_lines(err)
    return name, False, f"    exit {r.returncode} (want {code}); output differs\n" + first_lines(r.stdout.decode(errors="replace") + "\n--- want\n" + want.decode(errors="replace"), 8)


def phrases():
    out = {}
    for line in PHRASES.read_text().splitlines():
        if not line.strip() or line.startswith("#") or " :: " not in line:
            continue
        head, phrase = line.split(" :: ", 1)
        out[head.split()[0]] = phrase.strip()
    return out


def run_reject(item, table):
    name, path = item
    want = table.get(name)
    SCRATCH.mkdir(parents=True, exist_ok=True)
    r = subprocess.run([str(KAYO), "build", str(path), "-o", str(SCRATCH / name)], stdin=subprocess.DEVNULL, capture_output=True, timeout=120)
    err = r.stderr.decode(errors="replace")
    if r.returncode == 0:
        return name, False, "    accepted; the oracle says: " + (REJECT / f"{name}.err").read_text().strip()
    if want is None:
        return name, False, "    no phrase in reject_phrases.txt; kayo said:\n" + first_lines(err, 3)
    if want.startswith("!"):
        return name, True, ""
    if want in err:
        return name, True, ""
    return name, False, f"    wants: {want}\n    oracle: {(REJECT / (name + '.err')).read_text().strip()}\n" + first_lines(err, 4)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    reject = "--reject" in sys.argv
    if reject:
        table = phrases()
        items = targets(REJECT, args)
        work = lambda it: run_reject(it, table)
    else:
        items = targets(EXAMPLES, args)
        work = run_example
    with ThreadPoolExecutor(3) as pool:
        results = list(pool.map(work, items))
    ok = [n for n, good, _ in results if good]
    bad = [(n, why) for n, good, why in results if good is False]
    skipped = [(n, why) for n, good, why in results if good is None]
    for n, why in bad:
        print(f"{n}:\n{why}")
    for n, why in skipped:
        print(f"{n}: skipped ({why})")
    print(f"\n{len(ok)} pass, {len(bad)} fail, {len(skipped)} skipped, of {len(results)}")


if __name__ == "__main__":
    main()
