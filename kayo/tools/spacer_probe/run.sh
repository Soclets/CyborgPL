#!/bin/sh
# Z3's Spacer on the hand-encoded loops (owner, 2026-10-25: measure first), then each integer invariant it finds
# read in 64-bit bitvectors and checked by Z3 and CVC5. From this folder: sh run.sh
for f in pick sits sort; do
  for k in int bv; do
    start=$(python3 -c 'import time; print(time.time())')
    out=$(z3 -T:120 fp.engine=spacer rlimit=50000000 ${f}_$k.smt2 2>&1 | head -1)
    end=$(python3 -c 'import time; print(time.time())')
    echo "spacer $f $k: $out ($(python3 -c "print(round($end - $start, 2))") s)"
  done
done
for f in pick sits; do
  echo "confirm $f: z3 $(z3 -T:60 confirm_$f.smt2 2>&1 | head -1), cvc5 $(cvc5 --tlimit=60000 confirm_$f.smt2 2>&1 | head -1)"
done
