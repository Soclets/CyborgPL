import os, sys
try:
    n = int(os.environ.get("N", ""))
except ValueError:
    sys.exit(2)
if n < 0 or n > 200_000_000:
    sys.exit(2)
comp = bytearray(n + 1)
count = 0
for p in range(2, n + 1):
    if not comp[p]:
        count += 1
        for m in range(p * p, n + 1, p):
            comp[m] = 1
sys.exit(count % 256)
