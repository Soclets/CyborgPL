"""Every check site Precog proves, for every program the suites hold, so a change to Precog can be shown to prove
exactly the same sites (owner, 2026-10-25: a loop summarised only if no proof is lost).

    python3 -I kayo/tools/precog_sites.py record <label>        # from the repository's root, after building kayo
    python3 -I kayo/tools/precog_sites.py compare <before> <after>

`record` runs `kayo prove --json` on examples/ (files and kit folders), kayo/compiler/tests/programs/ and
check/corpus/, three at a time (the session's cores), and writes each program's sites, with the time it took, under
.kayo-checks/precog-sites/<label>/ (git-ignored). `compare` says, program by program, every site whose verdict
changed, and the totals and times of both."""
import json
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor

ROOT = os.getcwd()
KAYO = os.path.join(ROOT, "kayo/target/release/kayo")
OUT = os.path.join(ROOT, ".kayo-checks/precog-sites")


def programs():
    found = []
    for d in ["examples", "kayo/compiler/tests/programs", "check/corpus"]:
        for name in sorted(os.listdir(d)):
            path = os.path.join(d, name)
            if name.endswith(".cy") or (d == "examples" and os.path.isdir(path) and os.path.exists(os.path.join(path, "CyJet.toml"))):
                found.append(path)
    return found


def key(site):
    return "|".join(str(site.get(k)) for k in ["file", "start", "end", "function", "kind", "construct"])


def record_one(path):
    start = time.time()
    r = subprocess.run([KAYO, "prove", "--json", path], capture_output=True, text=True, timeout=3600)
    took = time.time() - start
    sites = []
    for line in r.stdout.splitlines():
        line = line.strip()
        if line.startswith("{"):
            try:
                s = json.loads(line)
            except json.JSONDecodeError:
                continue
            sites.append([key(s), s.get("verdict")])
    sites.sort()
    return path, {"exit": r.returncode, "seconds": round(took, 2), "sites": sites, "error": r.stderr[-400:] if r.returncode else ""}


def record(label):
    out = os.path.join(OUT, label)
    os.makedirs(out, exist_ok=True)
    progs = programs()
    with ThreadPoolExecutor(max_workers=3) as pool:
        for path, res in pool.map(record_one, progs):
            with open(os.path.join(out, path.replace("/", "__") + ".json"), "w") as f:
                json.dump(res, f)
    print(f"{len(progs)} programs recorded under {out}")


def load(label):
    d = os.path.join(OUT, label)
    return {n: json.load(open(os.path.join(d, n))) for n in sorted(os.listdir(d))}


def compare(before, after):
    a, b = load(before), load(after)
    lost = gained = changed_programs = 0
    ta = tb = 0.0
    counts = {before: {}, after: {}}
    for name in sorted(set(a) | set(b)):
        x, y = a.get(name), b.get(name)
        if x is None or y is None:
            print(f"{name}: only in {'after' if x is None else 'before'}")
            continue
        ta += x["seconds"]
        tb += y["seconds"]
        for label, r in [(before, x), (after, y)]:
            for _, v in r["sites"]:
                counts[label][v] = counts[label].get(v, 0) + 1
        if x["sites"] != y["sites"] or x["exit"] != y["exit"]:
            changed_programs += 1
            xs, ys = {}, {}
            for k, v in x["sites"]:
                xs.setdefault(k, []).append(v)
            for k, v in y["sites"]:
                ys.setdefault(k, []).append(v)
            print(f"{name}: exit {x['exit']} -> {y['exit']}")
            for k in sorted(set(xs) | set(ys)):
                if sorted(xs.get(k, [])) != sorted(ys.get(k, [])):
                    pb = xs.get(k, []).count("proven")
                    pa = ys.get(k, []).count("proven")
                    lost += max(0, pb - pa)
                    gained += max(0, pa - pb)
                    print(f"  {k}: {xs.get(k, [])} -> {ys.get(k, [])}")
    print(f"programs: {len(a)} before, {len(b)} after; {changed_programs} with any site changed")
    print(f"verdicts {before}: {dict(sorted(counts[before].items()))}")
    print(f"verdicts {after}: {dict(sorted(counts[after].items()))}")
    print(f"proofs lost: {lost}, gained: {gained}")
    print(f"time proving: {ta:.0f} s before, {tb:.0f} s after")
    slow = sorted(((a[n]["seconds"], b.get(n, {}).get("seconds"), n) for n in a), reverse=True)[:8]
    for s0, s1, n in slow:
        print(f"  {n}: {s0} s -> {s1} s")


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "record":
        record(sys.argv[2])
    elif len(sys.argv) == 4 and sys.argv[1] == "compare":
        compare(sys.argv[2], sys.argv[3])
    else:
        print(__doc__)
        sys.exit(2)
