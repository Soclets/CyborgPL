import os, sys
try:
    n = int(os.environ.get("N", ""))
except ValueError:
    sys.exit(2)
if n < 0 or n > 92:
    sys.exit(2)
sys.setrecursionlimit(10000)
def fib(k):
    return k if k < 2 else fib(k - 1) + fib(k - 2)
sys.exit(fib(n) % 256)
