#!/usr/bin/env python3
"""CCA against clang -O2 on the same programs.

Builds each `name.cpls` with CCA (`--final` when the final build accepts it,
the working build otherwise) and `name.c` with clang -O2, runs them in turn
with N set, keeps the best of several runs, and checks the answers agree.
Sizes are read at run time (N), so nothing is worked out while compiling.

    python3 run.py            # performance cores; close heavy apps first
    python3 run.py --ecores   # both sides pinned to efficiency cores
"""
import os, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
CCA = os.path.join(HERE, "..", "target", "release", "cca")
CASES = [("sum", 300_000_000), ("fib", 35), ("lists", 20_000_000), ("sieve", 100_000_000)]
RUNS = 7
PIN = ["taskpolicy", "-b"] if "--ecores" in sys.argv else []


def build(name):
    src = os.path.join(HERE, name + ".cpls")
    out = os.path.join(HERE, name + ".cca")
    t = time.perf_counter()
    r = subprocess.run([CCA, "build", "--final", src, "-o", out], capture_output=True, text=True)
    kind = "final"
    if r.returncode != 0:
        why = next((l for l in r.stderr.splitlines() if l.startswith("check:")), "refused")
        r = subprocess.run([CCA, "build", src, "-o", out], capture_output=True, text=True)
        kind = "working (" + why + ")"
    took = time.perf_counter() - t
    subprocess.run(["clang", "-O2", os.path.join(HERE, name + ".c"), "-o", os.path.join(HERE, name + ".c.bin")], check=True)
    return out, kind, took


def run(exe, n):
    t = time.perf_counter()
    r = subprocess.run(PIN + [exe], env={**os.environ, "N": str(n)})
    return time.perf_counter() - t, r.returncode


for name, n in CASES:
    exe, kind, took = build(name)
    c = os.path.join(HERE, name + ".c.bin")
    best_a = best_c = float("inf")
    agree = True
    for _ in range(RUNS):
        a, ra = run(exe, n)
        b, rb = run(c, n)
        best_a, best_c = min(best_a, a), min(best_c, b)
        agree &= ra == rb
    print(f"{name:6} N={n:<11} cca {best_a:7.3f} s  clang -O2 {best_c:7.3f} s  {best_a / best_c:5.2f}x  "
          f"{'agree' if agree else 'DIFFER'}  build {took:5.1f} s, {kind}")
