#!/usr/bin/env python3
"""Clang C (1 and 3 threads) against Metal on three kinds of GPU work.

Builds into ../build/gpu. Each program is run once untimed, then timed three
times from start to exit; the best is kept. Metal also reports the time its
GPU spent. Every answer is checked against C's on one thread.
"""
import os, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "build", "gpu")
CASES = [("sum", 300_000_000), ("map", 100_000_000), ("mandel", 4096)]
if "--quick" in sys.argv:
    CASES = [("sum", 3_000_000), ("map", 1_000_000), ("mandel", 256)]

def sh(*cmd):
    subprocess.run(cmd, check=True, cwd=HERE)

def build():
    os.makedirs(OUT, exist_ok=True)
    air = os.path.join(OUT, "kernels.air")
    # exact float math, so the GPU gives the CPU's answer (default is fast, rounded math)
    sh("xcrun", "metal", "-fmetal-math-mode=safe", "-c", "kernels.metal", "-o", air)
    sh("xcrun", "metallib", air, "-o", os.path.join(OUT, "kernels.metallib"))
    sh("clang", "-O2", "-fobjc-arc", "host.m", "-framework", "Metal", "-framework", "Foundation", "-o", os.path.join(OUT, "metal"))
    sh("clang", "-O2", "cpu.c", "-o", os.path.join(OUT, "cpu"))

def run(cmd, n):
    env = {**os.environ, "N": str(n), "METALLIB": os.path.join(OUT, "kernels.metallib")}
    best = gpu = answer = None
    for k in range(4):
        t = time.perf_counter()
        p = subprocess.run(cmd, env=env, capture_output=True, text=True, check=True)
        took = time.perf_counter() - t
        answer = p.stdout.strip()
        for line in p.stderr.splitlines():
            if line.startswith("gpu_ms"):
                g = float(line.split()[1])
                if k: gpu = g if gpu is None else min(gpu, g)
        if k: best = took if best is None else min(best, took)
    return best * 1000, gpu, answer

build()
print(f"| Test | C, 1 thread | C, 3 threads | Metal, whole program | Metal, GPU part |")
print(f"|---|---|---|---|---|")
for mode, n in CASES:
    c1 = run([os.path.join(OUT, "cpu"), mode, "1"], n)
    c3 = run([os.path.join(OUT, "cpu"), mode, "3"], n)
    mt = run([os.path.join(OUT, "metal"), mode], n)
    bad = [name for name, r in (("C 3 threads", c3), ("Metal", mt)) if r[2] != c1[2]]
    note = f" — WRONG: {', '.join(bad)}" if bad else ""
    print(f"| {mode} (N={n}) | {c1[0]:.1f} ms | {c3[0]:.1f} ms | {mt[0]:.1f} ms | {mt[1]:.2f} ms |{note}")
