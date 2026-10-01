# sieve, the proper Python way: a bytearray, each prime's multiples crossed off by one slice
# assignment, and the primes counted by bytearray.count in C.
import math
import os
import sys

try:
    n = int(os.environ["N"])
except (KeyError, ValueError):
    sys.exit(2)
if not 0 <= n <= 200_000_000:
    sys.exit(2)
if n < 2:
    sys.exit(0)
comp = bytearray(n + 1)
for p in range(2, math.isqrt(n) + 1):
    if not comp[p]:
        comp[p * p :: p] = b"\x01" * len(range(p * p, n + 1, p))
sys.exit((comp.count(0) - 2) % 256)
