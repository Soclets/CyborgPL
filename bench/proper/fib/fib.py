# fib: naive recursion, as the program asks.
import os
import sys


def fib(n):
    return n if n < 2 else fib(n - 1) + fib(n - 2)


try:
    n = int(os.environ["N"])
except (KeyError, ValueError):
    sys.exit(2)
if not 0 <= n <= 92:
    sys.exit(2)
sys.exit(fib(n) % 256)
