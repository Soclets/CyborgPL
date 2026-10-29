#!/usr/bin/env python3
"""Z3's Spacer in Precog, measured (owner, 2026-10-25: measure before deciding; the go, 2026-10-29).

    python3 -I kayo/tools/spacer_probe/measure.py [screen|time|all] [PROGRAM ...]    # from the repository's root

`screen` builds every example (files and kit folders) and every kayo test program once with Spacer on and
`KAYO_SPACER_LOG=1`, and keeps the programs where Spacer was asked about some loop: those are exactly the programs
with a site still unproven after the shapes, behind a loop. night_trains goes first. `time` builds each of those
three times with Spacer off (`KAYO_NO_SPACER=1`) and three times on, one build at a time (each build's layer 3
uses the session's 3 cores), and reads each build's report for its sites and its phase times; Spacer's log gives
each site it proved, the invariant, any that failed confirmation, and every loop that spent the budget. `all` does
both. Results go to kayo/tools/spacer_probe/measured.json and MEASURED.md.
"""
import json
import os
import re
import statistics
import subprocess
import sys
import tempfile

ROOT = os.getcwd()
KAYO = os.path.join(ROOT, "kayo/target/release/kayo")
HERE = os.path.join(ROOT, "kayo/tools/spacer_probe")
DATA = os.path.join(HERE, "measured.json")
OUT = tempfile.mkdtemp(prefix="kayo-spacer-")


def programs():
    found = ["examples/night_trains.cy"]
    for d in ["examples", "kayo/compiler/tests/programs"]:
        for name in sorted(os.listdir(d)):
            path = os.path.join(d, name)
            if path in found:
                continue
            if name.endswith(".cy") or (d == "examples" and os.path.isdir(path) and os.path.exists(os.path.join(path, "CyJet.toml"))):
                found.append(path)
    return found


def build(path, spacer, rlimit=None):
    env = dict(os.environ, KAYO_SPACER_LOG="1")
    env.pop("KAYO_SPACER_RLIMIT", None)
    if rlimit:
        env["KAYO_SPACER_RLIMIT"] = str(rlimit)
    env.pop("KAYO_NO_SPACER", None)
    env.pop("KAYO_STD", None)
    if not spacer:
        env["KAYO_NO_SPACER"] = "1"
    out = os.path.join(OUT, re.sub(r"[^A-Za-z0-9_]", "_", path))
    r = subprocess.run([KAYO, "build", path, "-o", out], capture_output=True, text=True, env=env, stdin=subprocess.DEVNULL)
    report = out + ".report.md"
    if r.returncode != 0 or not os.path.exists(report):
        return None
    text = open(report).read()
    os.remove(report)
    rows = [l.split(" | ") for l in text.splitlines() if l.startswith("| ") and l.count(" | ") >= 5 and ".cy:" in l.split(" | ")[0]]
    unproven = [r[0].strip("| ").strip() for r in rows if r[4].strip() == "checked at run time"]
    proven = sum(1 for r in rows if r[4].strip() == "proven")
    times = {}
    for m in re.finditer(r"^\| (\*\*Total\*\*|[A-Z][^|]*?) \| \**([0-9.,]+) (ms|s|µs)\**", text, re.M):
        n = float(m.group(2).replace(",", ""))
        n = {"s": n, "ms": n / 1e3, "µs": n / 1e6}[m.group(3)]
        times[m.group(1).strip("*")] = n
    log = [l[len("spacer: "):] for l in r.stderr.splitlines() if l.startswith("spacer: ")]
    return {"sites": len(rows), "proven": proven, "unproven": unproven, "precog": times.get("Precog"), "total": times.get("Total"), "log": log}


def screen(wanted):
    data = load()
    for path in wanted:
        b = build(path, True)
        if b is None:
            print(f"{path}: did not build", flush=True)
            continue
        print(f"{path}: {len(b['log'])} loop question(s)", flush=True)
        if b["log"]:
            data.setdefault(path, {})["screen"] = b["log"]
    save(data)


def time_all(wanted):
    data = load()
    for path in wanted:
        if path not in data:
            continue
        runs = {"off": [], "on": []}
        for _ in range(3):
            for mode in ("off", "on"):
                runs[mode].append(build(path, mode == "on"))
        d = data[path]
        for mode in ("off", "on"):
            rs = [r for r in runs[mode] if r]
            if len(rs) < 3:
                d[mode] = None
                continue
            d[mode] = {
                "sites": rs[0]["sites"],
                "unproven": rs[0]["unproven"],
                "same_every_run": all(r["unproven"] == rs[0]["unproven"] and r["log"] == rs[0]["log"] for r in rs),
                "precog": [r["precog"] for r in rs],
                "total": [r["total"] for r in rs],
                "log": rs[0]["log"],
            }
        save(data)
        print(f"{path}: off {med(d['off'], 'precog')} s Precog, on {med(d['on'], 'precog')} s", flush=True)


def med(d, k):
    return round(statistics.median(d[k]), 3) if d and all(x is not None for x in d[k]) else None


def load():
    return json.load(open(DATA)) if os.path.exists(DATA) else {}


def save(data):
    json.dump(data, open(DATA, "w"), indent=1, ensure_ascii=False)
    write_md(data)


def write_md(data):
    rlimit = re.search(r"SPACER_RLIMIT: u64 = ([0-9_]+)", open(os.path.join(ROOT, "kayo/compiler/src/precog/smt.rs")).read()).group(1)
    lines = ["# Spacer in Precog, measured", "",
             f"Made by `measure.py` (its docstring says how). Spacer's budget: `rlimit` {rlimit} of Z3's steps per loop "
             "question. Times are the median of three builds, one at a time, read from each build's report "
             "(the Precog phase, and the whole build).", ""]
    total_new = 0
    # The overview: one row a program, and the sums.
    rows = ["| program | unproven off | unproven on | Precog off, s | Precog on, s | build off, s | build on, s | questions | spent the budget |",
            "|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
    sums = [0.0, 0.0, 0.0, 0.0]
    count = [0, 0, 0, 0]
    for path, d in data.items():
        off, on = d.get("off"), d.get("on")
        if not off or not on:
            continue
        log = on["log"]
        nums = [med(off, "precog"), med(on, "precog"), med(off, "total"), med(on, "total")]
        sums = [a + b for a, b in zip(sums, nums)]
        count = [a + b for a, b in zip(count, [len(off["unproven"]), len(on["unproven"]), len(log), sum("spent its step budget" in l for l in log)])]
        rows.append(f"| {path} | {len(off['unproven'])} | {len(on['unproven'])} | {nums[0]} | {nums[1]} | {nums[2]} | {nums[3]} | {len(log)} | {sum('spent its step budget' in l for l in log)} |")
    rows.append(f"| **all {len(rows) - 2}** | **{count[0]}** | **{count[1]}** | **{sums[0]:.2f}** | **{sums[1]:.2f}** | **{sums[2]:.2f}** | **{sums[3]:.2f}** | **{count[2]}** | **{count[3]}** |")
    lines += ["## Every program Spacer was asked about", ""] + rows + [""]
    larger = sorted({k for d in data.values() for k in d if k.startswith("on@")})
    for k in larger:
        n = k[3:]
        lines += [f"## At a budget of {int(n):,} steps", "", "Only the programs where a loop question spent the default budget.", "",
                  "| program | unproven | Precog, s (median of 3) | build, s | questions that spent the budget | same every run |", "|---|---:|---:|---:|---:|---|"]
        for path, d in data.items():
            if k in d:
                e = d[k]
                lines.append(f"| {path} | {len(e['unproven'])} | {med(e, 'precog')} | {med(e, 'total')} | {sum('spent its step budget' in l for l in e['log'])} | {e['same_every_run']} |")
        lines.append("")
    for path, d in data.items():
        off, on = d.get("off"), d.get("on")
        if not off or not on:
            continue
        new = [s for s in off["unproven"] if s not in on["unproven"]]
        lost = [s for s in on["unproven"] if s not in off["unproven"]]
        total_new += len(new)
        log = on["log"]
        budget = [l for l in log if "spent its step budget" in l]
        loops = sorted({re.search(r"loop at line (\d+)", l).group(1) + " in " + re.search(r"\(in ([^)]*)\)", l).group(1) for l in log})
        loops_budget = sorted({re.search(r"loop at line (\d+)", l).group(1) + " in " + re.search(r"\(in ([^)]*)\)", l).group(1) for l in budget})
        lines += [f"## {path}", "",
                  "| | Spacer off | Spacer on |", "|---|---:|---:|",
                  f"| sites | {off['sites']} | {on['sites']} |",
                  f"| unproven (checked at run time) | {len(off['unproven'])} | {len(on['unproven'])} |",
                  f"| Precog, s (median of 3) | {med(off, 'precog')} | {med(on, 'precog')} |",
                  f"| whole build, s (median of 3) | {med(off, 'total')} | {med(on, 'total')} |",
                  f"| Precog, s, each run | {', '.join(str(x) for x in off['precog'])} | {', '.join(str(x) for x in on['precog'])} |",
                  f"| the same sites and log every run | {off['same_every_run']} | {on['same_every_run']} |",
                  f"| loop questions put to Spacer | | {len(log)} ({len(loops)} loops) |",
                  f"| questions that spent the budget | | {len(budget)} ({len(loops_budget)} loops) |", ""]
        if lost:
            lines += ["**Proven with Spacer off and not on**: " + ", ".join(lost), ""]
        proved = [l for l in log if "the site is proven" in l]
        if proved:
            lines += ["Newly proven:", ""] + [f"- {l}" for l in proved] + [""]
        failed = [l for l in log if "not confirmed" in l]
        if failed:
            lines += ["Failed confirmation:", ""] + [f"- {l}" for l in failed] + [""]
        lines += ["<details><summary>Spacer's whole log</summary>", "", "```"] + log + ["```", "</details>", ""]
    lines.insert(4, f"**Sites newly proven: {total_new}**, across the programs below.\n")
    open(os.path.join(HERE, "MEASURED.md"), "w").write("\n".join(lines) + "\n")


def budget(n, wanted):
    """Three builds with Spacer on at a budget of `n` steps, for the programs where a loop spent the default."""
    data = load()
    for path in wanted:
        d = data.get(path)
        if not d or not d.get("on") or not any("spent its step budget" in l for l in d["on"]["log"]):
            continue
        rs = [build(path, True, n) for _ in range(3)]
        if any(r is None for r in rs):
            continue
        d[f"on@{n}"] = {"sites": rs[0]["sites"], "unproven": rs[0]["unproven"], "same_every_run": all(r["unproven"] == rs[0]["unproven"] and r["log"] == rs[0]["log"] for r in rs), "precog": [r["precog"] for r in rs], "total": [r["total"] for r in rs], "log": rs[0]["log"]}
        save(data)
        print(f"{path} at {n}: {len(rs[0]['unproven'])} unproven, Precog {med(d[f'on@{n}'], 'precog')} s", flush=True)


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "all"
    if cmd == "budget":
        budget(int(sys.argv[2]), sys.argv[3:] or programs())
        sys.exit(0)
    wanted = sys.argv[2:] or programs()
    if cmd in ("screen", "all"):
        screen(wanted)
    if cmd in ("time", "all"):
        time_all(wanted)
