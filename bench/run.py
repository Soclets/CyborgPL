#!/usr/bin/env python3
"""CyborgPL against every other language installed here, on the same four programs, and n-body.

Each program reads N from the environment and exits with its answer's low byte, so
nothing is worked out while compiling and every language is checked against C's
answer. Lust has no getenv and no exit code, so the runner writes N into its source
and reads the printed answer instead. `nbody` (the Benchmarks Game's, in bench/nbody/) is
written only in C, C++, Rust, Zig, Fortran and CyborgPL; it prints the energy to 9 places
before and after, and its whole output must match C's text exactly. No version of it fuses
a multiply and an add: clang, clang++ and gfortran get -ffp-contract=off for it (CASE_FLAGS),
and rustc, Zig and kayo do not fuse unless asked. A run is timed from process start to exit,
startup included; each case keeps the best of RUNS, and a run past TIMEOUT seconds
is recorded as a timeout.

    python3 run.py                     # every language that is installed
    python3 run.py c rust cyborgpl          # only these
    python3 run.py --quick             # tiny sizes, one run: does everything build and agree?
    python3 run.py --allow-working     # time a working build where kayo refuses the final one
    KAYO=/path/to/kayo LUST=/path/to/lust python3 run.py
    CYBORGPL_BENCH_THREADS=10 python3 run.py   # CyborgPL's row on every core, when nothing else runs

CyborgPL is timed only as a final build. If kayo refuses one (a check it cannot prove), that case is
reported as refused and not timed, unless --allow-working is given; then the working build, which
keeps run-time checks, is timed and its row says so. The run ends with a summary of anything that
did not agree, was refused, or fell back. The tables are written to latest.md; RESULTS.md is the
dated history, kept by hand.
"""
import os, platform, re, shutil, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
BUILD = os.path.join(HERE, "build")
CASES = [("sum", 300_000_000), ("fib", 35), ("lists", 20_000_000), ("sieve", 100_000_000), ("nbody", 100_000_000)]
RUNS = 3
TIMEOUT = 60
ALLOW_WORKING = "--allow-working" in sys.argv
if ALLOW_WORKING:
    sys.argv.remove("--allow-working")
if "--quick" in sys.argv:  # tiny sizes, one run: checks every language builds and agrees
    sys.argv.remove("--quick")
    CASES = [("sum", 100_000), ("fib", 20), ("lists", 100_000), ("sieve", 100_000), ("nbody", 1_000)]
    RUNS = 1
_KAYO = os.path.join(HERE, "..", "kayo", "target", "release", "kayo")
KAYO = os.environ.get("KAYO", _KAYO)
LUST = os.environ.get("LUST", "lust")
# Cases written in only some languages; every other case is in all of them.
CASE_LANGS = {"nbody": {"cyborgpl", "cyborgpl1", "cyborgpl_nc", "cyborgpl1_nc", "c", "cpp", "rust", "zig", "fortran"}}
# Cases checked by what they print (all of stdout, exit code 0) instead of by exit code.
PRINTS = {"nbody"}
# Flags a case adds to a compiler's usual ones, by the command's name: nbody's no-FMA rule.
CASE_FLAGS = {"nbody": {"clang": ["-ffp-contract=off"], "clang++": ["-ffp-contract=off"], "gfortran": ["-ffp-contract=off"]}}


def has_case(key, name):
    return key in CASE_LANGS.get(name, (key,))


def label_for(key, name):
    """The language's label, with the flags this case adds to its build."""
    label, tool = LANGS[key][0], LANGS[key][1]
    extra = CASE_FLAGS.get(name, {}).get(tool)
    return label[:-1] + " " + " ".join(extra) + ")" if extra and label.endswith(")") else label


def sh(cmd, cwd=None):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(" ".join(cmd) + "\n" + (r.stderr or r.stdout)[-2000:])


def cap(name):
    return name[0].upper() + name[1:]


def dotnet_major():
    v = subprocess.run(["dotnet", "--version"], capture_output=True, text=True).stdout.strip()
    return v.split(".")[0]


# Each language: (label, tool that must be installed, source name, build, run).
# build(src, out_dir) returns nothing; run(src, out_dir) returns the command to time.
def native(cmd):
    def build(src, d, b):
        c = cmd(src, os.path.join(d, b))
        sh(c + CASE_FLAGS.get(b, {}).get(c[0], []))
    return build


# kayo's final builds it refused, by program, with the first line of why (2026-10-15: a refused sieve
# once fell back silently to a working build and passed as a final one)
REFUSED = {}


class Refused(Exception):
    pass


def kayo_build(src, d, b):
    out = os.path.join(d, b)
    r = subprocess.run([KAYO, "build", "--final", src, "-o", out], capture_output=True, text=True)
    if r.returncode == 0:
        return
    lines = [l.strip() for l in (r.stderr or r.stdout).splitlines() if l.strip()]
    why = " / ".join(lines[:2]) or "refused"
    REFUSED[b] = why
    if not ALLOW_WORKING:
        raise Refused(f"final build refused: {why}")
    print(f"  ! {b}: kayo refused the final build ({why}); timing a WORKING build, as --allow-working asks", flush=True)
    sh([KAYO, "build", src, "-o", out])


def ocaml_build(src, d, b):
    shutil.copy(src, os.path.join(d, b + ".ml"))
    sh(["ocamlfind", "ocamlopt", "-O3", "-o", b, b + ".ml"] if shutil.which("ocamlfind") else ["ocamlopt", "-o", b, b + ".ml"], cwd=d)


def dotnet_build(src, d, b):
    proj = os.path.join(d, "cs")
    os.makedirs(proj, exist_ok=True)
    shutil.copy(src, os.path.join(proj, "Program.cs"))
    with open(os.path.join(proj, b + ".csproj"), "w") as f:
        f.write('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType>'
                f'<TargetFramework>net{dotnet_major()}.0</TargetFramework><Optimize>true</Optimize>'
                '<InvariantGlobalization>true</InvariantGlobalization><TieredPGO>true</TieredPGO>'
                '</PropertyGroup></Project>')
    sh(["dotnet", "build", "-c", "Release", "-o", os.path.join(d, "cs-out"), "--nologo", "-v", "q"], cwd=proj)


def ts_build(src, d, b):
    sh(["tsc", "--target", "es2020", "--outDir", d, src])


LANGS = {
    "cyborgpl": (f"CyborgPL (kayo, final build, {os.environ.get('CYBORGPL_BENCH_THREADS', os.environ.get('CYBORG_BENCH_THREADS', '3'))} threads)", None, "{b}.cy", kayo_build, lambda s, d, b: [os.path.join(d, b)]),
    "cyborgpl1": ("CyborgPL (kayo, final build, 1 thread)", None, "{b}.cy", kayo_build, lambda s, d, b: [os.path.join(d, b)]),
    # `no_cheat` in front of the work (owner, 2026-10-19: "with cheat and no_cheat"): every round really runs
    "cyborgpl_nc": (f"CyborgPL no_cheat (kayo, final build, {os.environ.get('CYBORGPL_BENCH_THREADS', os.environ.get('CYBORG_BENCH_THREADS', '3'))} threads)", None, "{b}_no_cheat.cy", kayo_build, lambda s, d, b: [os.path.join(d, b)]),
    "cyborgpl1_nc": ("CyborgPL no_cheat (kayo, final build, 1 thread)", None, "{b}_no_cheat.cy", kayo_build, lambda s, d, b: [os.path.join(d, b)]),
    "c": ("C (clang -O3)", "clang", "{b}.c", native(lambda s, o: ["clang", "-O3", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "asm": ("Assembly (AArch64, hand-written)", "clang", "{b}.s", native(lambda s, o: ["clang", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "cpp": ("C++ (clang++ -O3)", "clang++", "{b}.cpp", native(lambda s, o: ["clang++", "-O3", "-std=c++17", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "rust": ("Rust (rustc opt-level=3)", "rustc", "{b}.rs", native(lambda s, o: ["rustc", "-C", "opt-level=3", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "go": ("Go", "go", "{b}.go", native(lambda s, o: ["go", "build", "-o", o, s]), lambda s, d, b: [os.path.join(d, b)]),
    "zig": ("Zig (ReleaseFast)", "zig", "{b}.zig", native(lambda s, o: ["zig", "build-exe", "-O", "ReleaseFast", "-femit-bin=" + o, s]), lambda s, d, b: [os.path.join(d, b)]),
    # Odin (owner, 2026-10-21: "Bench it against CyborgPL, Zig, and C only"): its release build, bounds checks kept as by default
    "odin": ("Odin (-o:speed)", "odin", "{b}.odin", native(lambda s, o: ["odin", "build", s, "-file", "-o:speed", "-out:" + o]), lambda s, d, b: [os.path.join(d, b)]),
    "swift": ("Swift (-O)", "swiftc", "{b}.swift", native(lambda s, o: ["swiftc", "-O", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "nim": ("Nim (release)", "nim", "{b}.nim", native(lambda s, o: ["nim", "c", "-d:release", "--hints:off", "--nimcache:" + o + ".cache", "-o:" + o, s]), lambda s, d, b: [os.path.join(d, b)]),
    "fortran": ("Fortran (gfortran -O3)", "gfortran", "{b}.f90", native(lambda s, o: ["gfortran", "-O3", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "ocaml": ("OCaml (ocamlopt)", "ocamlopt", "{b}.ml", ocaml_build, lambda s, d, b: [os.path.join(d, b)]),
    "dart": ("Dart (AOT)", "dart", "{b}.dart", native(lambda s, o: ["dart", "compile", "exe", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "java": ("Java", "javac", "{B}.java", native(lambda s, o: ["javac", "-d", os.path.dirname(o), s]), lambda s, d, b: ["java", "-cp", d, cap(b)]),
    "kotlin": ("Kotlin (JVM)", "kotlinc", "{b}.kt", native(lambda s, o: ["kotlinc", s, "-include-runtime", "-d", o + ".jar"]), lambda s, d, b: ["java", "-jar", os.path.join(d, b + ".jar")]),
    "csharp": ("C# (.NET)", "dotnet", "{b}.cs", dotnet_build, lambda s, d, b: ["dotnet", os.path.join(d, "cs-out", b + ".dll")]),
    "javascript": ("JavaScript (Node)", "node", "{b}.js", None, lambda s, d, b: ["node", s]),
    "typescript": ("TypeScript (tsc + Node)", "tsc", "{b}.ts", ts_build, lambda s, d, b: ["node", os.path.join(d, b + ".js")]),
    "julia": ("Julia", "julia", "{b}.jl", None, lambda s, d, b: ["julia", "--startup-file=no", s]),
    "luajit": ("LuaJIT", "luajit", "{b}.lua", None, lambda s, d, b: ["luajit", s]),
    "pypy": ("PyPy", "pypy3", "{b}.py", None, lambda s, d, b: ["pypy3", s]),
    "elixir": ("Elixir", "elixir", "{b}.exs", None, lambda s, d, b: ["elixir", s]),
    "erlang": ("Erlang (escript)", "escript", "{b}.erl", None, lambda s, d, b: ["escript", s]),
    "lust": ("Lust", None, "{b}.lust", None, lambda s, d, b: [LUST, os.path.join(d, b + ".lust")]),
    "python": ("Python (CPython)", "python3", "{b}.py", None, lambda s, d, b: ["python3", s]),
    "ruby": ("Ruby", "ruby", "{b}.rb", None, lambda s, d, b: ["ruby", "--yjit", s]),
    "lua": ("Lua", "lua", "{b}.lua", None, lambda s, d, b: ["lua", s]),
    "perl": ("Perl", "perl", "{b}.pl", None, lambda s, d, b: ["perl", s]),
    "r": ("R", "Rscript", "{b}.R", None, lambda s, d, b: ["Rscript", "--vanilla", s]),
}


def available(key):
    label, tool, *_ = LANGS[key]
    if key.startswith("cyborgpl"):
        return os.path.exists(KAYO)
    if key == "lust":
        return shutil.which(LUST) is not None or os.path.exists(LUST)
    return shutil.which(tool) is not None


# CyborgPL spreads a `for` over the cores by itself (parallelism.md); every other language runs
# its plain code. It gets this session's share, 3 threads, and a second row pins it to 1.
# `CYBORG_BENCH_THREADS=10 python3 run.py` gives it every core, when nothing else is running
# (owner, 2026-10-06: "uncapped")
CYBORGPL_THREADS = os.environ.get("CYBORGPL_BENCH_THREADS", os.environ.get("CYBORG_BENCH_THREADS", "3"))
THREADS = {"cyborgpl": CYBORGPL_THREADS, "cyborgpl1": "1", "cyborgpl_nc": CYBORGPL_THREADS, "cyborgpl1_nc": "1"}


def run_once(cmd, n, lust, key="", prints=False):
    env = dict(os.environ, N=str(n))
    if key in THREADS:
        env["THREAD_CAP"] = THREADS[key]; env["CYBORG_THREADS"] = THREADS[key]  # until every build reads THREAD_CAP
    t = time.perf_counter()
    try:
        r = subprocess.run(cmd, env=env, capture_output=True, text=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return None, None
    took = time.perf_counter() - t
    if prints:
        return took, (r.stdout if r.returncode == 0 else ("fail", r.returncode))
    if lust:
        m = re.search(r"-?\d+", r.stdout)
        return took, (int(m.group()) if m and r.returncode == 0 else ("fail", r.returncode))
    return took, r.returncode


def kayo_version():
    try:
        return subprocess.run([KAYO, "version"], capture_output=True, text=True).stdout.strip().splitlines()[0]
    except Exception:
        return "unknown version"


# The main set (owner, 2026-10-21): what a plain `python3 run.py` runs; `all` runs every language above,
# and naming languages runs those. CyborgPL's rows: 3 threads and 1, each normal and no_cheat.
MAIN_SET = ["cyborgpl", "cyborgpl1", "cyborgpl_nc", "cyborgpl1_nc", "c", "cpp", "fortran", "zig", "rust", "go", "java", "python"]


def main():
    args = [a.lower() for a in sys.argv[1:]]
    want = list(LANGS) if args == ["all"] else (args or MAIN_SET)
    keys = [k for k in LANGS if k in want and available(k)]
    missing = [k for k in LANGS if k in want and not available(k)]
    if any(k.startswith("cyborgpl") for k in keys):
        print(f"kayo: {KAYO} ({kayo_version()}); CyborgPL's own row on {CYBORGPL_THREADS} threads; "
              f"{'working builds allowed where the final build is refused' if ALLOW_WORKING else 'final builds only'}", flush=True)
    print(f"Best of {RUNS} runs each, checked against C's answer.\n", flush=True)
    os.makedirs(BUILD, exist_ok=True)
    expect = {}  # the C answer for each case
    for name, n in CASES:
        d = os.path.join(BUILD, "c", name)
        os.makedirs(d, exist_ok=True)
        LANGS["c"][3](os.path.join(HERE, name, name + ".c"), d, name)
        r = subprocess.run([os.path.join(d, name)], env=dict(os.environ, N=str(n)), capture_output=True, text=True)
        expect[name] = (r.stdout if r.returncode == 0 else ("fail", r.returncode)) if name in PRINTS else r.returncode
    results = {}
    for key in keys:
        label, tool, pattern, build, run = LANGS[key]
        for name, n in CASES:
            if not has_case(key, name):
                continue
            src = os.path.join(HERE, name, pattern.format(b=name, B=cap(name)))
            d = os.path.join(BUILD, key, name)
            os.makedirs(d, exist_ok=True)
            try:
                if key == "lust":
                    with open(src) as f, open(os.path.join(d, name + ".lust"), "w") as g:
                        g.write(f.read().replace("{{N}}", str(n)))
                if build:
                    build(src, d, name)
            except Refused as e:
                results[key, name] = ("refused", str(e))
                print(f"{key:11} {name:6} REFUSED   {e}", flush=True)
                continue
            except Exception as e:
                results[key, name] = ("build failed", str(e).splitlines()[0][:120])
                print(f"{key:11} {name:6} build failed", flush=True)
                continue
            best, answer = None, None
            if build:  # warm-up, untimed: macOS scans a freshly built program on its first run
                run_once(run(src, d, name), n, key == "lust", key, name in PRINTS)
            for _ in range(RUNS):
                took, got = run_once(run(src, d, name), n, key == "lust", key, name in PRINTS)
                if took is None:
                    best = None if best is None else best
                    answer = "timeout" if best is None else answer
                    break
                answer = got
                best = took if best is None else min(best, took)
                if took > 10:
                    break
            ok = answer == expect[name]
            shown_answer = repr if name in PRINTS else str  # printed text, with its newlines shown
            results[key, name] = (best, "ok" if ok else f"answer {shown_answer(answer)}, C says {shown_answer(expect[name])}")
            shown = f"{best:8.3f} s" if best is not None else "  timeout"
            print(f"{key:11} {name:6} {shown}  {'agree' if ok else results[key, name][1]}", flush=True)
    write_results(keys, missing, results)
    summary(results)


def summary(results):
    """What a reader must know before trusting the numbers: what disagreed, was refused, or fell back."""
    wrong = [(k, n, st) for (k, n), (t, st) in results.items() if isinstance(t, float) and st != "ok"]
    refused = [(k, n, st) for (k, n), (t, st) in results.items() if t == "refused"]
    failed = [(k, n, st) for (k, n), (t, st) in results.items() if t == "build failed"]
    fell_back = sorted(REFUSED) if ALLOW_WORKING else []
    print("\nSummary:")
    if not (wrong or refused or failed or fell_back):
        print("  every case built as asked and agreed with C. Tables in latest.md.")
        return
    for k, n, st in wrong: print(f"  WRONG ANSWER  {k} {n}: {st}")
    for k, n, st in refused: print(f"  REFUSED       {k} {n}: {st} (not timed)")
    for n in fell_back: print(f"  WORKING BUILD cyborgpl {n}: final build refused ({REFUSED[n]}); its rows are marked")
    for k, n, st in failed: print(f"  BUILD FAILED  {k} {n}: {st}")
    print("  Tables in latest.md.")


def write_results(keys, missing, results):
    lines = ["# Benchmark results", "",
             f"Machine: {platform.machine()}, {platform.system()} {platform.release()}; "
             f"run {time.strftime('%Y-%m-%d')} by `bench/run.py`. Best of {RUNS} runs, wall-clock "
             f"from process start to exit (startup included); runs past {TIMEOUT} s are timeouts. "
             "Every answer is checked against C's.", ""]
    for name, n in CASES:
        lines += [f"## {name} (N = {n:,})", "", "| Language | Time | vs CyborgPL |", "|---|---|---|"]
        base = results.get(("cyborgpl", name), (None,))[0]
        rows = []
        for key in keys:
            if not has_case(key, name):
                continue
            t, status = results.get((key, name), (None, "not run"))
            if isinstance(t, float) and status == "ok":
                rel = f"{t / base:.2f}×" if isinstance(base, float) else "—"
                label = label_for(key, name)
                if key.startswith("cyborgpl") and name in REFUSED:
                    label = label.replace("final build", "**working build**: the final build was refused") 
                rows.append((t, f"| {label} | {t:.3f} s | {rel} |"))
            else:
                why = "timeout" if status == "ok" or t is None and status.startswith("answer timeout") else status
                rows.append((float("inf"), f"| {label_for(key, name)} | {why} | — |"))
        lines += [r for _, r in sorted(rows, key=lambda x: x[0])] + [""]
    if missing:
        lines += ["Not installed, so not run: " + ", ".join(LANGS[k][0] for k in missing) + ".", ""]
    with open(os.path.join(HERE, "latest.md"), "w") as f:
        f.write("\n".join(lines))


if __name__ == "__main__":
    main()
