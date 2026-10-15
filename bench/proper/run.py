#!/usr/bin/env python3
"""The proper benchmark: Zag, C, Rust and Go, each written the way that language's own practice
says, with only what the language comes with, in two versions per program: proper (one thread, what
an expert writes first) and parallel (every core, with the language's own threads: pthreads,
std::thread::scope, goroutines, Zag's `for para` and `concur`). Zag is timed only as a final build.

Each program reads N from the environment and exits with its answer's low byte; every answer is
checked against C's. Times are wall-clock from start to exit, best of RUNS, after one untimed warm-up.

    python3 run.py               # Zag, C, Rust and Go, proper and parallel
    python3 run.py c zag         # only these (java, python, julia, luajit: proper only)
    python3 run.py --quick       # tiny sizes, one run: does everything build and agree?

The tables go to latest.md; RESULTS.md is the dated history, kept by hand.
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
ZOMP = os.environ.get("ZOMP", os.path.join(HERE, "..", "..", "zomp", "target", "release", "zomp"))
REFUSED = {}


class Refused(Exception):
    pass


def sh(cmd, cwd=None):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(" ".join(cmd) + "\n" + (r.stderr or r.stdout)[-2000:])


def cap(name):
    return name[0].upper() + name[1:]


def build(lang, name, para):
    """The command that runs `name` in `lang`: its proper version, or its `_para` one."""
    stem = name + ("_para" if para else "")
    d = os.path.join(BUILD, lang + ("_para" if para else ""), name)
    os.makedirs(d, exist_ok=True)
    src = os.path.join(HERE, name)
    out = os.path.join(d, name)
    if lang == "c":
        sh(["clang", "-O3", os.path.join(src, stem + ".c"), "-o", out])
        return [out]
    if lang == "rust":
        sh(["rustc", "-C", "opt-level=3", os.path.join(src, stem + ".rs"), "-o", out])
        return [out]
    if lang == "go":
        sh(["go", "build", "-o", out, os.path.join(src, stem + ".go")])
        return [out]
    if lang == "zag":
        r = subprocess.run([ZOMP, "build", "--final", os.path.join(src, stem + ".zg"), "-o", out], capture_output=True, text=True)
        if r.returncode != 0:
            why = " / ".join(l.strip() for l in (r.stderr or r.stdout).splitlines() if l.strip())[:200]
            REFUSED[stem] = why
            raise Refused(f"final build refused: {why}")
        return [out]
    if para:
        raise ValueError(f"no parallel {lang}")
    if lang == "java":
        sh(["javac", "-d", d, os.path.join(src, cap(name) + ".java")])
        return ["java", "-cp", d, cap(name)]
    if lang == "python":
        return ["python3", os.path.join(src, name + ".py")]
    if lang == "julia":
        return ["julia", "-t", "auto", "--startup-file=no", os.path.join(src, name + ".jl")]
    if lang == "luajit":
        return ["luajit", os.path.join(src, name + ".lua")]
    raise ValueError(lang)


LABELS = {"zag": "Zag (Zomp, final build)", "c": "C (clang -O3)", "rust": "Rust (rustc opt-level=3)", "go": "Go",
          "java": "Java", "python": "Python (CPython)", "julia": "Julia (-t auto)", "luajit": "LuaJIT"}
DEFAULT = ["zag", "c", "rust", "go"]


def run(cmd, n, threads=None):
    env = dict(os.environ, N=str(n))
    env.pop("THREAD_CAP", None)
    env.pop("CYBORG_THREADS", None)
    if threads:
        env["THREAD_CAP"] = threads
    t = time.perf_counter()
    r = subprocess.run(cmd, env=env, capture_output=True)
    return time.perf_counter() - t, r.returncode


def main():
    want = [a.lower() for a in sys.argv[1:]] or DEFAULT
    langs = [l for l in LABELS if l in want]
    print(f"Zomp: {ZOMP}. Proper versions single-threaded (Zag capped to 1 thread, and also shown uncapped, "
          f"as written); parallel versions on every core. Final builds only.\n", flush=True)
    expect = {name: run(build("c", name, False), n)[1] for name, n in CASES}
    # (row label, lang, para, THREAD_CAP)
    rows_plan = []
    for lang in langs:
        if lang == "zag":
            rows_plan += [("Zag, proper, 1 thread", lang, False, "1"), ("Zag, proper, as written (Zomp splits proven loops itself)", lang, False, None)]
        else:
            rows_plan.append((f"{LABELS[lang]}, proper", lang, False, None))
    for lang in langs:
        if lang in ("zag", "c", "rust", "go"):
            rows_plan.append((f"{LABELS[lang].split(' (')[0]}, parallel", lang, True, None))
    results = {}
    for label, lang, para, threads in rows_plan:
        for name, n in CASES:
            try:
                cmd = build(lang, name, para)
            except Refused as e:
                results[label, name] = (None, str(e)); print(f"{label:58} {name:6} REFUSED  {e}", flush=True); continue
            except Exception as e:
                results[label, name] = (None, "build failed: " + str(e).splitlines()[0][:100]); print(f"{label:58} {name:6} build failed", flush=True); continue
            run(cmd, n, threads)  # warm-up, untimed
            best, got = None, None
            for _ in range(RUNS):
                took, got = run(cmd, n, threads)
                best = took if best is None else min(best, took)
            ok = got == expect[name]
            results[label, name] = (best, "ok" if ok else f"answer {got}, C says {expect[name]}")
            print(f"{label:58} {name:6} {best:8.3f} s  {'agree' if ok else results[label, name][1]}", flush=True)
    lines = ["# The proper benchmark: latest run", "",
             f"{platform.machine()}, {platform.system()} {platform.release()}; run {time.strftime('%Y-%m-%d %H:%M')} "
             f"by `bench/proper/run.py`. Best of {RUNS} after a warm-up, wall-clock from start to exit. "
             "Every answer is checked against C's.", ""]
    for name, n in CASES:
        lines += [f"## {name} (N = {n:,})", "", "| Version | Time |", "|---|---|"]
        rows = []
        for label, *_ in rows_plan:
            t, status = results.get((label, name), (None, "not run"))
            rows.append((t if t is not None and status == "ok" else float("inf"), f"| {label} | {f'{t:.3f} s' if t is not None and status == 'ok' else status} |"))
        lines += [r for _, r in sorted(rows)] + [""]
    with open(os.path.join(HERE, "latest.md"), "w") as f:
        f.write("\n".join(lines))
    bad = [(k, v) for k, v in results.items() if v[1] != "ok"]
    print("\nSummary:", "every case built as asked and agreed with C. Tables in latest.md." if not bad else "")
    for (label, name), (_, st) in bad:
        print(f"  {label} {name}: {st}")


if __name__ == "__main__":
    main()
