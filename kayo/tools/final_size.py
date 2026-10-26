"""The size and speed of final builds (owner, 2026-10-25: a final build carries only what the program uses, unless
it costs the final product's speed or safety).

    python3 -I kayo/tools/final_size.py <kayo> <label>           # from the repository's root
    python3 -I kayo/tools/final_size.py compare <before> <after>

Builds hello world and the bench and example programs below with `<kayo> build --final`, and writes each one's
size (as built and stripped), its sites (proven / checked), and the best of five runs' time, under
.kayo-checks/final-size/<label>.json (git-ignored). `compare` prints both side by side."""
import json
import os
import shutil
import subprocess
import sys
import time

ROOT = os.getcwd()
OUT = os.path.join(ROOT, ".kayo-checks/final-size")
HELLO = 'MAIN {\n    std::print.useful("hello, world", newln);\n}\n'
PROGRAMS = ["hello", "bench/sum/sum.cy", "bench/lists/lists.cy", "bench/fib/fib.cy", "bench/sieve/sieve.cy", "bench/nbody/nbody.cy", "examples/night_trains.cy", "examples/print_format_width.cy", "examples/msl_twice.cy"]


def build(kayo, prog, work):
    src = prog
    if prog == "hello":
        src = os.path.join(work, "hello.cy")
        open(src, "w").write(HELLO)
    exe = os.path.join(work, os.path.basename(prog).replace(".cy", ""))
    r = subprocess.run([kayo, "build", src, "--final", "-o", exe], capture_output=True, text=True, timeout=3600)
    if r.returncode != 0:
        return {"error": (r.stdout + r.stderr)[-300:]}
    line = (r.stdout + r.stderr).strip().splitlines()[-1]
    stripped = exe + ".stripped"
    shutil.copy(exe, stripped)
    subprocess.run(["strip", stripped])
    best = None
    for _ in range(5):
        t = time.time()
        subprocess.run([exe], capture_output=True, timeout=600)
        took = time.time() - t
        best = took if best is None else min(best, took)
    return {"bytes": os.path.getsize(exe), "stripped": os.path.getsize(stripped), "built": line, "best_run_s": round(best, 4)}


def record(kayo, label):
    work = os.path.join(OUT, label)
    os.makedirs(work, exist_ok=True)
    out = {p: build(os.path.abspath(kayo), p, work) for p in PROGRAMS if p == "hello" or os.path.exists(p)}
    json.dump(out, open(os.path.join(OUT, label + ".json"), "w"), indent=1)
    for p, r in out.items():
        print(p, r)


def compare(a, b):
    x = json.load(open(os.path.join(OUT, a + ".json")))
    y = json.load(open(os.path.join(OUT, b + ".json")))
    print(f"| program | {a} bytes | {b} bytes | {a} stripped | {b} stripped | {a} best run | {b} best run | sites |")
    print("|---|---:|---:|---:|---:|---:|---:|---|")
    for p in x:
        u, v = x[p], y.get(p, {})
        sites = lambda r: r.get("built", "").split(": ")[1].split(", ")[0:3] if ": " in r.get("built", "") else []
        same = "same" if sites(u) == sites(v) else f"{sites(u)} -> {sites(v)}"
        print(f"| {p} | {u.get('bytes')} | {v.get('bytes')} | {u.get('stripped')} | {v.get('stripped')} | {u.get('best_run_s')} | {v.get('best_run_s')} | {same} |")


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "compare":
        compare(sys.argv[2], sys.argv[3])
    elif len(sys.argv) == 3:
        record(sys.argv[1], sys.argv[2])
    else:
        print(__doc__)
        sys.exit(2)
