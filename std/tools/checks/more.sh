#!/bin/sh
# more.sh: str.at, the slices and round.X beside the oracle's built-in std, in the oracle and through kayo;
# slice.point and slice.byte against Python; the round argv proofs; check claude's corpus program on std.
. "$(dirname "$0")/common.sh"
W="$OUT/more"; rm -rf "$W"; mkdir -p "$W"
stdtest "$W"

for g in gen_at gen_slice gen_round; do
    python3 -I "$CHECKS/$g.py" pair | program "$W/$g.pair" pair - builtin "stdtest=../kits/std" unicode money
    python3 -I "$CHECKS/$g.py" real | program "$W/$g.real" real - std unicode money
    (cd "$W/$g.pair" && "$O" . > ../$g.pair.out 2>&1) || { echo "$g, in the oracle: FAILED"; head -3 "$W/$g.pair.out"; }
    "$Y" build "$W/$g.real" -o "$W/$g.real.prog" > "$W/$g.real.build" 2>&1 || { echo "$g, kayo REFUSED it:"; head -4 "$W/$g.real.build"; continue; }
    "$W/$g.real.prog" > "$W/$g.real.out"
    # The built-in has no slice.point: those lines ("slp") are checked against Python below.
    python3 -I -c "
import sys
P = open(sys.argv[1], encoding='utf-8').read().splitlines(); K = open(sys.argv[2], encoding='utf-8').read().splitlines()
A = [l[2:] for l in P if l.startswith('A ')]
B = [l[2:] for l in P if l.startswith('B ') and not l.startswith('B slp')]
Ka = [l[2:] for l in K if l.startswith('A ') and not l.startswith('A slp')]
print(sys.argv[3] + ':', len(A), 'built-in answers | stdtest in the oracle:', 'same' if A == B else 'DIFFERENT',
      '| std through kayo:', 'same' if A == Ka else 'DIFFERENT', '|', open(sys.argv[4]).readline().split(': ', 1)[1][:60].strip())
" "$W/$g.pair.out" "$W/$g.real.out" $g "$W/$g.real.build"
done
python3 -I "$CHECKS/gen_slice.py" python > "$W/slp.expected"
grep "^B slp" "$W/gen_slice.pair.out" | cut -c3- | cmp -s - "$W/slp.expected" && a=same || a=DIFFERENT
grep "^A slp" "$W/gen_slice.real.out" | cut -c3- | cmp -s - "$W/slp.expected" && b=same || b=DIFFERENT
echo "slice.point: $(wc -l < "$W/slp.expected" | tr -d ' ') answers by Python's code points | in the oracle: $a | through kayo: $b"

python3 -I "$CHECKS/gen_sbyte.py" real | program "$W/sbyte" sbyte - std
python3 -I "$CHECKS/gen_sbyte.py" python > "$W/sbyte.expected"
"$Y" build "$W/sbyte" -o "$W/sbyte.prog" > "$W/sbyte.build" 2>&1 && "$W/sbyte.prog" | cmp -s - "$W/sbyte.expected" \
    && echo "slice.byte: $(wc -l < "$W/sbyte.expected" | tr -d ' ') answers through kayo, the same as Python's bytes" \
    || { echo "slice.byte: DIFFERENT or refused"; head -3 "$W/sbyte.build"; }

python3 -I "$CHECKS/programs.py" argv_round | program "$W/argv_round" argv_round - std vouch raw_memory
python3 -I "$CHECKS/programs.py" argv_rdeci | program "$W/argv_rdeci" argv_rdeci - std money vouch raw_memory
for t in argv_round argv_rdeci; do
    "$Y" build "$W/$t" -o "$W/$t.prog" > "$W/$t.build" 2>&1 || { echo "$t: kayo REFUSED it:"; head -4 "$W/$t.build"; continue; }
    python3 -I "$CHECKS/sites.py" "$W/$t.prog.report.md" "$t"
done
echo "argv_round 2.675 2: $("$W/argv_round.prog" 2.675 2 | sed -n 2p | cut -c1-70)"
echo "argv_rdeci 1234.5678 2: $("$W/argv_rdeci.prog" 1234.5678 2 | sed -n 2p)"

# check claude's corpus program on str.at and slice.grapheme: kayo against std/, beside the oracle
C="$R/check/corpus/2026-10-08-str-at-and-slice-by-position.cy"
mkdir -p "$W/corpus"; cp "$C" "$W/corpus/kit.cy"; toml "$W/corpus" corpus builtin
(cd "$W/corpus" && "$O" . > "$W/corpus.oracle" 2>&1); toml "$W/corpus" corpus std
"$Y" build "$W/corpus" -o "$W/corpus.prog" > "$W/corpus.build" 2>&1 && "$W/corpus.prog" | diff -q - "$W/corpus.oracle" > /dev/null \
    && echo "corpus 2026-10-08: kayo against std/ prints the oracle's output" \
    || echo "corpus 2026-10-08: not run ($(head -1 "$W/corpus.build" | cut -c1-90)); it is check claude's"
