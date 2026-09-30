import os, sys
try:
    n = int(os.environ.get("N", ""))
except ValueError:
    sys.exit(2)
if n < 0 or n > 100_000_000:
    sys.exit(2)
xs = []
for i in range(1, n + 1):
    xs.append(i % 1000)
acc = 0
for r in range(10):
    for j in range(len(xs)):
        acc += xs[j]
sys.exit(acc % 256)
