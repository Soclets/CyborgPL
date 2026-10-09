#!/usr/bin/env python3
"""CyborgPL against every other language installed here, on the same four programs.

Each program reads N from the environment and exits with its answer's low byte, so
nothing is worked out while compiling and every language is checked against C's
answer. Lust has no getenv and no exit code, so the runner writes N into its source
and reads the printed answer instead. A run is timed from process start to exit,
startup included; each case keeps the best of RUNS, and a run past TIMEOUT seconds
is recorded as a timeout.

    python3 run.py                   # every language that is installed
    python3 run.py c rust nerve   # only these
    python3 run.py --quick           # tiny sizes, one run: does everything build and agree?
    SYNAPSE=/path/to/synapse LUST=/path/to/lust python3 run.py

The results are written to RESULTS.md.
"""
import os, platform, re, shutil, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
BUILD = os.path.join(HERE, "build")
CASES = [("sum", 300_000_000), ("fib", 35), ("lists", 20_000_000), ("sieve", 100_000_000)]
RUNS = 3
TIMEOUT = 60
if "--quick" in sys.argv:  # tiny sizes, one run: checks every language builds and agrees
    sys.argv.remove("--quick")
    CASES = [("sum", 100_000), ("fib", 20), ("lists", 100_000), ("sieve", 100_000)]
    RUNS = 1
SYNAPSE = os.environ.get("SYNAPSE", os.path.join(HERE, "..", "synapse", "target", "release", "synapse"))
LUST = os.environ.get("LUST", "lust")


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
    return lambda src, d, b: sh(cmd(src, os.path.join(d, b)))


def synapse_build(src, d, b):
    out = os.path.join(d, b)
    r = subprocess.run([SYNAPSE, "build", "--final", src, "-o", out], capture_output=True, text=True)
    if r.returncode != 0:
        sh([SYNAPSE, "build", src, "-o", out])


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
    "nerve": (f"Nerve (Synapse, final build, {os.environ.get('CYBORG_BENCH_THREADS', '3')} threads)", None, "{b}.nrv", synapse_build, lambda s, d, b: [os.path.join(d, b)]),
    "nerve1": ("Nerve (Synapse, final build, 1 thread)", None, "{b}.nrv", synapse_build, lambda s, d, b: [os.path.join(d, b)]),
    "c": ("C (clang -O2)", "clang", "{b}.c", native(lambda s, o: ["clang", "-O2", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "asm": ("Assembly (AArch64, hand-written)", "clang", "{b}.s", native(lambda s, o: ["clang", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "cpp": ("C++ (clang++ -O2)", "clang++", "{b}.cpp", native(lambda s, o: ["clang++", "-O2", "-std=c++17", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "rust": ("Rust (rustc -O)", "rustc", "{b}.rs", native(lambda s, o: ["rustc", "-C", "opt-level=3", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "go": ("Go", "go", "{b}.go", native(lambda s, o: ["go", "build", "-o", o, s]), lambda s, d, b: [os.path.join(d, b)]),
    "zig": ("Zig (ReleaseFast)", "zig", "{b}.zig", native(lambda s, o: ["zig", "build-exe", "-O", "ReleaseFast", "-femit-bin=" + o, s]), lambda s, d, b: [os.path.join(d, b)]),
    "swift": ("Swift (-O)", "swiftc", "{b}.swift", native(lambda s, o: ["swiftc", "-O", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
    "nim": ("Nim (release)", "nim", "{b}.nim", native(lambda s, o: ["nim", "c", "-d:release", "--hints:off", "--nimcache:" + o + ".cache", "-o:" + o, s]), lambda s, d, b: [os.path.join(d, b)]),
    "fortran": ("Fortran (gfortran -O2)", "gfortran", "{b}.f90", native(lambda s, o: ["gfortran", "-O2", s, "-o", o]), lambda s, d, b: [os.path.join(d, b)]),
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
    if key.startswith("nerve"):
        return os.path.exists(SYNAPSE)
    if key == "lust":
        return shutil.which(LUST) is not None or os.path.exists(LUST)
    return shutil.which(tool) is not None


# CyborgPL spreads a `for` over the cores by itself (parallelism.md); every other language runs
# its plain code. It gets this session's share, 3 threads, and a second row pins it to 1.
# `CYBORG_BENCH_THREADS=10 python3 run.py` gives it every core, when nothing else is running
# (owner, 2026-10-06: "uncapped")
THREADS = {"nerve": os.environ.get("CYBORG_BENCH_THREADS", "3"), "nerve1": "1"}


def run_once(cmd, n, lust, key=""):
    env = dict(os.environ, N=str(n))
    if key in THREADS:
        env["CYBORG_THREADS"] = THREADS[key]
    t = time.perf_counter()
    try:
        r = subprocess.run(cmd, env=env, capture_output=True, text=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return None, None
    took = time.perf_counter() - t
    if lust:
        m = re.search(r"-?\d+", r.stdout)
        return took, (int(m.group()) if m and r.returncode == 0 else ("fail", r.returncode))
    return took, r.returncode


def main():
    want = [a.lower() for a in sys.argv[1:]] or list(LANGS)
    keys = [k for k in LANGS if k in want and available(k)]
    missing = [k for k in LANGS if k in want and not available(k)]
    os.makedirs(BUILD, exist_ok=True)
    expect = {}  # the C answer for each case
    for name, n in CASES:
        d = os.path.join(BUILD, "c", name)
        os.makedirs(d, exist_ok=True)
        LANGS["c"][3](os.path.join(HERE, name, name + ".c"), d, name)
        expect[name] = subprocess.run([os.path.join(d, name)], env=dict(os.environ, N=str(n))).returncode
    results = {}
    for key in keys:
        label, tool, pattern, build, run = LANGS[key]
        for name, n in CASES:
            src = os.path.join(HERE, name, pattern.format(b=name, B=cap(name)))
            d = os.path.join(BUILD, key, name)
            os.makedirs(d, exist_ok=True)
            try:
                if key == "lust":
                    with open(src) as f, open(os.path.join(d, name + ".lust"), "w") as g:
                        g.write(f.read().replace("{{N}}", str(n)))
                if build:
                    build(src, d, name)
            except Exception as e:
                results[key, name] = ("build failed", str(e).splitlines()[0][:120])
                print(f"{key:11} {name:6} build failed", flush=True)
                continue
            best, answer = None, None
            if build:  # warm-up, untimed: macOS scans a freshly built program on its first run
                run_once(run(src, d, name), n, key == "lust", key)
            for _ in range(RUNS):
                took, got = run_once(run(src, d, name), n, key == "lust", key)
                if took is None:
                    best = None if best is None else best
                    answer = "timeout" if best is None else answer
                    break
                answer = got
                best = took if best is None else min(best, took)
                if took > 10:
                    break
            ok = answer == expect[name]
            results[key, name] = (best, "ok" if ok else f"answer {answer}, C says {expect[name]}")
            shown = f"{best:8.3f} s" if best is not None else "  timeout"
            print(f"{key:11} {name:6} {shown}  {'agree' if ok else results[key, name][1]}", flush=True)
    write_results(keys, missing, results)


def write_results(keys, missing, results):
    lines = ["# Benchmark results", "",
             f"Machine: {platform.machine()}, {platform.system()} {platform.release()}; "
             f"run {time.strftime('%Y-%m-%d')} by `bench/run.py`. Best of {RUNS} runs, wall-clock "
             f"from process start to exit (startup included); runs past {TIMEOUT} s are timeouts. "
             "Every answer is checked against C's.", ""]
    for name, n in CASES:
        lines += [f"## {name} (N = {n:,})", "", "| Language | Time | vs CyborgPL |", "|---|---|---|"]
        base = results.get(("nerve", name), (None,))[0]
        rows = []
        for key in keys:
            t, status = results.get((key, name), (None, "not run"))
            if isinstance(t, float) and status == "ok":
                rel = f"{t / base:.2f}×" if isinstance(base, float) else "—"
                rows.append((t, f"| {LANGS[key][0]} | {t:.3f} s | {rel} |"))
            else:
                why = "timeout" if status == "ok" or t is None and status.startswith("answer timeout") else status
                rows.append((float("inf"), f"| {LANGS[key][0]} | {why} | — |"))
        lines += [r for _, r in sorted(rows, key=lambda x: x[0])] + [""]
    if missing:
        lines += ["Not installed, so not run: " + ", ".join(LANGS[k][0] for k in missing) + ".", ""]
    with open(os.path.join(HERE, "RESULTS.md"), "w") as f:
        f.write("\n".join(lines))


if __name__ == "__main__":
    main()
