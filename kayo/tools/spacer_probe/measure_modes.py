#!/usr/bin/env python3
"""Spacer's modes side by side (owner, 2026-10-29: "Test out number 2 first": integer clauses, confirmed in
bit-vectors).

    python3 -I kayo/tools/spacer_probe/measure_modes.py [PROGRAM ...]    # from the repository's root

Takes the programs the first measurement found asking Spacer anything (measured.json), and two_counters.cy, and
builds each three times in each mode, one build at a time (each build's layer 3 uses the session's 3 cores):

    off            KAYO_NO_SPACER=1
    bits           Spacer over bit-vectors, the default
    integers       KAYO_SPACER_INTEGERS=1
    integers+gg    KAYO_SPACER_INTEGERS=1 KAYO_SPACER_GLOBAL=1 (Spacer's global guidance)
    joint          KAYO_SPACER_INTEGERS=1 KAYO_SPACER_JOINT=1 (every loop on the way as one system)
    joint+gg       the same, with global guidance
    bits+gg        KAYO_SPACER_GLOBAL=1

Each build's report gives its unproven sites and phase times, and Spacer's log each question's outcome. Results go to
kayo/tools/spacer_probe/measured_modes.json and MODES.md.
"""
import json
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import measure  # noqa: E402

ROOT = os.getcwd()
HERE = os.path.join(ROOT, "kayo/tools/spacer_probe")
DATA = os.path.join(HERE, "measured_modes.json")
MODES = [
    ("off", False, {}),
    ("bits", True, {}),
    ("integers", True, {"KAYO_SPACER_INTEGERS": "1"}),
    ("integers+gg", True, {"KAYO_SPACER_INTEGERS": "1", "KAYO_SPACER_GLOBAL": "1"}),
    ("joint", True, {"KAYO_SPACER_INTEGERS": "1", "KAYO_SPACER_JOINT": "1"}),
    ("joint+gg", True, {"KAYO_SPACER_INTEGERS": "1", "KAYO_SPACER_JOINT": "1", "KAYO_SPACER_GLOBAL": "1"}),
    ("bits+gg", True, {"KAYO_SPACER_GLOBAL": "1"}),
]


def programs():
    first = json.load(open(os.path.join(HERE, "measured.json")))
    return list(first) + ["kayo/tools/spacer_probe/two_counters.cy"]


def med(xs):
    return round(statistics.median(xs), 3) if xs and all(x is not None for x in xs) else None


def run(wanted):
    data = json.load(open(DATA)) if os.path.exists(DATA) else {}
    for path in wanted:
        d = data.setdefault(path, {})
        for name, on, extra in MODES:
            rs = [measure.build(path, on, extra=extra) for _ in range(3)]
            if any(r is None for r in rs):
                d[name] = None
                continue
            d[name] = {
                "sites": rs[0]["sites"],
                "unproven": rs[0]["unproven"],
                "same_every_run": all(r["unproven"] == rs[0]["unproven"] and r["log"] == rs[0]["log"] for r in rs),
                "precog": [r["precog"] for r in rs],
                "total": [r["total"] for r in rs],
                "log": rs[0]["log"],
            }
        json.dump(data, open(DATA, "w"), indent=1, ensure_ascii=False)
        write_md(data)
        print(path + ": " + ", ".join(f"{n} {len(d[n]['unproven']) if d.get(n) else '-'} unproven {med(d[n]['precog']) if d.get(n) else '-'} s" for n, _, _ in MODES), flush=True)


def write_md(data):
    names = [n for n, _, _ in MODES]
    rlimit = re.search(r"SPACER_RLIMIT: u64 = ([0-9_]+)", open(os.path.join(ROOT, "kayo/compiler/src/precog/smt.rs")).read()).group(1)
    lines = ["# Spacer's modes, measured", "",
             f"Made by `measure_modes.py` (its docstring says how). Budget: `rlimit` {rlimit} of Z3's steps per loop question, "
             "in every mode. Times are the median of three builds, one at a time, from each build's report.", ""]
    head = "| program | " + " | ".join(f"unproven {n}" for n in names) + " | " + " | ".join(f"Precog {n}, s" for n in names) + " |"
    lines += ["## Every program", "", head, "|---|" + "---:|" * (2 * len(names))]
    sums = {n: [0, 0.0, 0.0, 0, 0] for n in names}
    for path, d in data.items():
        if not all(d.get(n) for n in names):
            lines.append(f"| {path} | did not build in some mode |" + " |" * (2 * len(names) - 1))
            continue
        row = [str(len(d[n]["unproven"])) for n in names] + [str(med(d[n]["precog"])) for n in names]
        lines.append(f"| {path} | " + " | ".join(row) + " |")
        for n in names:
            s = sums[n]
            s[0] += len(d[n]["unproven"])
            s[1] += med(d[n]["precog"])
            s[2] += med(d[n]["total"])
            s[3] += len(d[n]["log"])
            s[4] += sum("spent its step budget" in l for l in d[n]["log"])
    lines.append("| **all** | " + " | ".join(f"**{sums[n][0]}**" for n in names) + " | " + " | ".join(f"**{sums[n][1]:.2f}**" for n in names) + " |")
    lines += ["", "| mode | unproven | Precog, s | whole builds, s | loop questions | spent the budget |", "|---|---:|---:|---:|---:|---:|"]
    for n in names:
        s = sums[n]
        lines.append(f"| {n} | {s[0]} | {s[1]:.2f} | {s[2]:.2f} | {s[3]} | {s[4]} |")
    lines.append("")
    for path, d in data.items():
        if not all(d.get(n) for n in names):
            continue
        lines += [f"## {path}", "", "| | " + " | ".join(names) + " |", "|---|" + "---:|" * len(names)]
        lines.append("| unproven | " + " | ".join(str(len(d[n]["unproven"])) for n in names) + " |")
        lines.append("| Precog, s | " + " | ".join(str(med(d[n]["precog"])) for n in names) + " |")
        lines.append("| whole build, s | " + " | ".join(str(med(d[n]["total"])) for n in names) + " |")
        lines.append("| spent the budget | " + " | ".join(str(sum("spent its step budget" in l for l in d[n]["log"])) for n in names) + " |")
        lines.append("| the same every run | " + " | ".join(str(d[n]["same_every_run"]) for n in names) + " |")
        lines.append("")
        for n in names[1:]:
            new = [s for s in d["off"]["unproven"] if s not in d[n]["unproven"]]
            lost = [s for s in d[n]["unproven"] if s not in d["off"]["unproven"]]
            proved = [l for l in d[n]["log"] if "the site is proven" in l]
            failed = [l for l in d[n]["log"] if "not confirmed" in l]
            if new or lost or proved or failed:
                lines.append(f"**{n}**: {len(new)} newly proven" + (f", **{len(lost)} lost**" if lost else ""))
                lines += [f"- proven: {l}" for l in proved] + [f"- rejected by bit-vector confirmation: {l}" for l in failed] + [""]
        lines += ["<details><summary>Spacer's logs</summary>", ""]
        for n in names[1:]:
            lines += [f"{n}:", "```"] + d[n]["log"] + ["```"]
        lines += ["</details>", ""]
    open(os.path.join(HERE, "MODES.md"), "w").write("\n".join(lines) + "\n")


if __name__ == "__main__":
    run(sys.argv[1:] or programs())
