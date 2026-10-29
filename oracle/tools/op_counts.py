#!/usr/bin/env python3
"""How often operators are used in real code, for the redesign checklist's frequency count (design/redesign-checklist.md,
step 2; first used 2026-10-28 for power against XOR). Counts lines, with grep-like patterns, so the figures are rough:

  python3 oracle/tools/op_counts.py            # Rust crates under ~/.cargo/registry (first 4000 files), Python's stdlib
"""
import glob, os, re, sysconfig

RUST = {
    "power (.pow/.powi/.powf)": r"\.(pow|powi|powf|checked_pow|wrapping_pow)\(",
    "times (a * b)": r"[a-zA-Z0-9_)\]] \* [a-zA-Z0-9_(]",
    "plus (a + b)": r"[a-zA-Z0-9_)\]] \+ [a-zA-Z0-9_(]",
    "xor (a ^ b)": r"[a-zA-Z0-9_)\]] \^ [a-zA-Z0-9_(]",
    "shift (<< >>)": r"[a-zA-Z0-9_)\]] (<<|>>) [a-zA-Z0-9_(]",
}
PYTHON = {
    "power (** or pow()": r"[^*(,] ?\*\* ?[^*=]|\bpow\(",
    "times (*)": r"[a-zA-Z0-9_)\]] ?\* ?[a-zA-Z0-9_(]",
    "plus (+)": r"[a-zA-Z0-9_)\]] ?\+ ?[a-zA-Z0-9_(]",
    "xor (^)": r"[a-zA-Z0-9_)\]] \^ [a-zA-Z0-9_(]",
}


def lines(files, comment):
    for f in files:
        with open(f, encoding="utf-8", errors="replace") as fh:
            for line in fh:
                if not line.lstrip().startswith(comment):
                    yield line


def count(name, files, comment, pats, skip=None):
    ls = list(lines(files, comment))
    print(f"{name}: {len(ls):,} lines")
    for label, p in pats.items():
        r = re.compile(p)
        n = sum(1 for l in ls if r.search(l) and not (skip and skip.search(l)))
        print(f"  {label}: {n:,}")


reg = sorted(glob.glob(os.path.expanduser("~/.cargo/registry/src/*/")))
if reg:
    rs = [f for f in sorted(glob.glob(os.path.join(reg[0], "**", "*.rs"), recursive=True)) if "/tests/" not in f][:4000]
    count("Rust crates", rs, "//", RUST)
std = sysconfig.get_paths()["stdlib"]
py = [f for f in glob.glob(os.path.join(std, "*.py")) + glob.glob(os.path.join(std, "*", "*.py")) if "/test" not in f]
count("Python stdlib", py, "#", PYTHON, skip=re.compile(r"\*\*kw|\*\*\w*args|\(\*\*"))
