#!/usr/bin/env python3
"""The proper benchmark: C, Rust, Go, Java, Python, Julia and CyborgPL, each written the way
that language's own practice says, with only what the language comes with. Threads are
not capped: a version uses every core if its language's usual tools do.

Each program reads N from the environment and exits with its answer's low byte; every
answer is checked against C's. Times are wall-clock from start to exit, best of RUNS,
after one untimed warm-up run.

    python3 run.py              # all seven
    python3 run.py c cyborgpl   # only these
    python3 run.py --quick      # tiny sizes, one run: does everything build and agree?

Results go to RESULTS.md next to this file.
"""
import os, platform, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
BUILD = os.path.join(HERE, "..", "build", "proper")
CASES = [("sum", 300_000_000), ("fib", 35), ("lists", 20_000_000), ("sieve", 100_000_000)]
RUNS = 3
if "--quick" in sys.argv:
    sys.argv.remove("--quick")
    CASES = [("sum", 100_000), ("fib", 20), ("lists", 100_000), ("sieve", 100_000)]
    RUNS = 1
CCA = os.environ.get("CCA", os.path.join(HERE, "..", "..", "cca", "target", "release", "cca"))


def sh(cmd, cwd=None):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(" ".join(cmd) + "\n" + r.stderr[-2000:])


def cap(name):
    return name[0].upper() + name[1:]


def build(lang, name):
    d = os.path.join(BUILD, lang, name)
    os.makedirs(d, exist_ok=True)
    src = os.path.join(HERE, name)
    out = os.path.join(d, name)
    if lang == "c":
        sh(["clang", "-O2", os.path.join(src, name + ".c"), "-o", out])
        return [out]
    if lang == "rust":
        sh(["rustc", "-C", "opt-level=3", os.path.join(src, name + ".rs"), "-o", out])
        return [out]
    if lang == "go":
        sh(["go", "build", "-o", out, os.path.join(src, name + ".go")])
        return [out]
    if lang == "java":
        sh(["javac", "-d", d, os.path.join(src, cap(name) + ".java")])
        return ["java", "-cp", d, cap(name)]
    if lang == "python":
        return ["python3", os.path.join(src, name + ".py")]
    if lang == "julia":
        # Threads are off unless asked for; `-t auto` is how Julia uses every core.
        return ["julia", "-t", "auto", "--startup-file=no", os.path.join(src, name + ".jl")]
    if lang == "luajit":
        return ["luajit", os.path.join(src, name + ".lua")]
    if lang == "cyborgpl":
        sh([CCA, "build", "--final", os.path.join(src, name + ".nrv"), "-o", out])
        return [out]
    raise ValueError(lang)


LABELS = {"c": "C (clang -O2)", "rust": "Rust (rustc, opt-level 3)", "go": "Go", "java": "Java",
          "python": "Python (CPython)", "julia": "Julia (-t auto)", "luajit": "LuaJIT", "cyborgpl": "CyborgPL (CCA, final build)"}


def run(cmd, n):
    env = dict(os.environ, N=str(n))
    env.pop("CYBORG_THREADS", None)  # uncapped: CyborgPL uses every core, as the others may
    t = time.perf_counter()
    r = subprocess.run(cmd, env=env, capture_output=True)
    return time.perf_counter() - t, r.returncode


def main():
    want = [a.lower() for a in sys.argv[1:]] or list(LABELS)
    langs = [l for l in LABELS if l in want]
    expect = {name: run(build("c", name), n)[1] for name, n in CASES}
    results = {}
    for lang in langs:
        for name, n in CASES:
            try:
                cmd = build(lang, name)
            except Exception as e:
                results[lang, name] = (None, "build failed: " + str(e).splitlines()[0][:100])
                print(f"{lang:9} {name:6} build failed", flush=True)
                continue
            run(cmd, n)  # warm-up, untimed
            best, got = None, None
            for _ in range(RUNS):
                took, got = run(cmd, n)
                best = took if best is None else min(best, took)
            ok = got == expect[name]
            results[lang, name] = (best, "ok" if ok else f"answer {got}, C says {expect[name]}")
            print(f"{lang:9} {name:6} {best:8.3f} s  {'agree' if ok else results[lang, name][1]}", flush=True)
    lines = ["# The proper benchmark: results", "",
             f"{platform.machine()}, {platform.system()} {platform.release()}; run {time.strftime('%Y-%m-%d')} "
             f"by `bench/proper/run.py`. Best of {RUNS} after a warm-up, wall-clock from start to exit. "
             "Every answer is checked against C's.", ""]
    for name, n in CASES:
        lines += [f"## {name} (N = {n:,})", "", "| Language | Time | vs CyborgPL |", "|---|---|---|"]
        base = results.get(("cyborgpl", name), (None,))[0]
        rows = []
        for lang in langs:
            t, status = results.get((lang, name), (None, "not run"))
            if t is not None and status == "ok":
                rel = f"{t / base:.2f}×" if base else "—"
                rows.append((t, f"| {LABELS[lang]} | {t:.3f} s | {rel} |"))
            else:
                rows.append((float("inf"), f"| {LABELS[lang]} | {status} | — |"))
        lines += [r for _, r in sorted(rows)] + [""]
    with open(os.path.join(HERE, "RESULTS.md"), "w") as f:
        f.write("\n".join(lines))


if __name__ == "__main__":
    main()
