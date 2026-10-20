#!/usr/bin/env python3
"""How long each language takes to build the four bench programs, from source to a final executable.

Asked for the website's "Don't script, don't iterate, but fucking wait." section (owner, through website claude,
2026-10-19). Best of 3 wall-clock builds per program, every one cold: Go and Zig get a fresh, empty cache directory
for each build, so nothing compiled earlier is reused (Go's and Zig's standard libraries included); clang, rustc,
gfortran and kayo keep no cache between runs. Each build runs alone, one after another.

    python3 compile_time.py                  # every language below (Python, the main set's ninth, has no build)
    python3 compile_time.py cyborgpl c rust  # some of them
"""
import os, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
KAYO = os.environ.get("KAYO", os.path.join(HERE, "..", "kayo", "target", "release", "kayo"))
PROGRAMS = ["sum", "fib", "lists", "sieve"]
RUNS = 3

# (label, source ending, command maker(src, out, scratch) -> (argv, extra env))
LANGS = {
    "cyborgpl": ("CyborgPL (kayo build --final)", "cy", lambda s, o, t: ([KAYO, "build", "--final", s, "-o", o], {})),
    "c": ("C (clang -O3)", "c", lambda s, o, t: (["clang", "-O3", s, "-o", o], {})),
    "rust": ("Rust (rustc -C opt-level=3)", "rs", lambda s, o, t: (["rustc", "-C", "opt-level=3", s, "-o", o], {})),
    "go": ("Go (go build, empty cache)", "go", lambda s, o, t: (["go", "build", "-o", o, s], {"GOCACHE": os.path.join(t, "gocache")})),
    "zig": ("Zig (ReleaseFast, empty cache)", "zig", lambda s, o, t: (["zig", "build-exe", "-O", "ReleaseFast", "-femit-bin=" + o, "--cache-dir", os.path.join(t, "zc"), "--global-cache-dir", os.path.join(t, "zg"), s], {})),
    "fortran": ("Fortran (gfortran -O3)", "f90", lambda s, o, t: (["gfortran", "-O3", s, "-o", o], {})),
    "cpp": ("C++ (clang++ -O3)", "cpp", lambda s, o, t: (["clang++", "-O3", "-std=c++17", s, "-o", o], {})),
    # Java's build is javac to class files, its JIT the rest at run time; the output is a directory of them
    "java": ("Java (javac)", "java", lambda s, o, t: (["javac", "-d", o, s], {})),
}


def build_once(lang, prog):
    label, ext, make = LANGS[lang]
    src = os.path.join(HERE, prog, f"{prog[0].upper() + prog[1:] if ext == 'java' else prog}.{ext}")
    with tempfile.TemporaryDirectory(prefix="ctime-") as t:
        out = os.path.join(t, prog)
        argv, env = make(src, out, t)
        start = time.perf_counter()
        r = subprocess.run(argv, cwd=t, env={**os.environ, **env}, capture_output=True, text=True)
        took = time.perf_counter() - start
        if r.returncode != 0 or not os.path.exists(out) or (os.path.isdir(out) and not os.listdir(out)):
            raise SystemExit(f"{lang} {prog}: the build failed:\n{(r.stderr or r.stdout)[-2000:]}")
        return took


def main():
    want = [a for a in sys.argv[1:]] or list(LANGS)
    if "cyborgpl" in want:
        v = subprocess.run([KAYO, "version"], capture_output=True, text=True).stdout.strip().splitlines()[:1]
        print(f"kayo: {KAYO} ({v[0] if v else '?'})")
    print(f"Best of {RUNS} cold builds each, wall-clock, seconds.\n")
    results = {}
    for lang in want:
        if lang not in LANGS:
            raise SystemExit(f"no language `{lang}`; there are {', '.join(LANGS)}")
        row = {}
        for prog in PROGRAMS:
            row[prog] = min(build_once(lang, prog) for _ in range(RUNS))
            print(f"{lang:10} {prog:6} {row[prog]:8.3f} s", flush=True)
        results[lang] = row
    print("\n| Language | " + " | ".join(PROGRAMS) + " | total |")
    print("|---|" + "---|" * (len(PROGRAMS) + 1))
    for lang, row in results.items():
        total = sum(row.values())
        print(f"| {LANGS[lang][0]} | " + " | ".join(f"{row[p]:.2f} s" for p in PROGRAMS) + f" | {total:.2f} s |")


if __name__ == "__main__":
    main()
