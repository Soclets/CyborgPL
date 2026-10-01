# lists, the proper Python way: a list comprehension, summed by the built-in sum().
import os
import sys

try:
    n = int(os.environ["N"])
except (KeyError, ValueError):
    sys.exit(2)
if not 0 <= n <= 100_000_000:
    sys.exit(2)
xs = [i % 1000 for i in range(1, n + 1)]
acc = sum(sum(xs) for _ in range(10))
sys.exit(acc % 256)
