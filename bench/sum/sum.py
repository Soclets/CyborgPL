import os, sys
try:
    n = int(os.environ.get("N", ""))
except ValueError:
    sys.exit(2)
if n < 0 or n > 1_000_000_000:
    sys.exit(2)
acc = 0
for i in range(1, n + 1):
    acc = (acc + (i * i) % 7) % 1_000_000_007
sys.exit(acc % 256)
