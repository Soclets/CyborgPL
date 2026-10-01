# sum, the proper Python way: CPU-bound work goes to a process pool, one worker per core.
import os
import sys
from multiprocessing import Pool


def part(bounds):
    lo, hi = bounds
    return sum(i * i % 7 for i in range(lo, hi + 1))


if __name__ == "__main__":
    try:
        n = int(os.environ["N"])
    except (KeyError, ValueError):
        sys.exit(2)
    if not 0 <= n <= 1_000_000_000:
        sys.exit(2)
    k = os.cpu_count() or 1
    with Pool(k) as pool:
        acc = sum(pool.map(part, [(1 + n * t // k, n * (t + 1) // k) for t in range(k)]))
    sys.exit(acc % 1_000_000_007 % 256)
