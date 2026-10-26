#!/bin/sh
# check.sh: std's main checks. Build kayo and the oracle first (`cargo build --release -j 3` in each), then
# std/tools/checks/check.sh; more.sh, case.sh and sweeps.sh are the rest. Results in .std-checks/work/.
. "$(dirname "$0")/common.sh"
W="$OUT/work"; rm -rf "$W"; mkdir -p "$W"
stdtest "$W"

# 1. argument_words, unchanged, against std/, and a final build of it
program "$W/aw" aw "$R/examples/argument_words.cy" std
"$Y" build "$W/aw" -o "$W/aw.prog" > "$W/aw.build" 2>&1 || { cat "$W/aw.build"; exit 1; }
"$Y" build --final "$W/aw" -o "$W/aw.final" > "$W/aw.final.build" 2>&1 && echo "argument_words: a final build accepts it" \
    || { echo "argument_words: a final build REFUSES it:"; head -5 "$W/aw.final.build"; }
"$W/aw.prog" | diff -q - "$R/examples/argument_words.out" > /dev/null && echo "argument_words: prints its .out ($(first_line "$W/aw.build"))" \
    || echo "argument_words: DIFFERS from its .out"

# 2. the text functions, the oracle's built-in std beside stdtest, in the oracle
python3 -I "$CHECKS/gen_text.py" pair | program "$W/pair" pair - builtin "stdtest=../kits/std"
(cd "$W/pair" && "$O" . > "$W/pair.out" 2>&1) || { echo "oracle failed:"; head -5 "$W/pair.out"; exit 1; }
python3 -I -c "
import sys
n = bad = 0
for line in open(sys.argv[1], encoding='utf-8'):
    line = line.rstrip('\n')
    if ' ||| ' not in line: bad += 1; print('??', line); continue
    head, b = line.split(' ||| ', 1); a = head.split(' ', 2)[2]; n += 1
    if a != b: bad += 1; print('DIFF', repr(line))
print('text, in the oracle:', n, 'cases,', bad, 'differ from its built-in std')" "$W/pair.out"

# 3. the same through kayo against std/
python3 -I "$CHECKS/gen_text.py" real | program "$W/real" real - std
"$Y" build "$W/real" -o "$W/real.prog" > "$W/real.build" 2>&1 || { cat "$W/real.build"; exit 1; }
"$W/real.prog" > "$W/real.out"
python3 -I -c "
import sys
exp = [l.rstrip('\n').split(' ||| ', 1)[0] for l in open(sys.argv[1], encoding='utf-8')]
got = [l.rstrip('\n') for l in open(sys.argv[2], encoding='utf-8')]
print('text, through kayo:', len(got), 'lines,', sum(a != b for a, b in zip(exp, got)) + abs(len(exp) - len(got)), 'differ from the oracle\'s built-in std')" "$W/pair.out" "$W/real.out"

# 4. text from the command line: what Precog proves from std's own code. It must say it ran the program
#    "over what its inputs allow", or the proofs only cover the inputs it ran.
python3 -I "$CHECKS/programs.py" argv | program "$W/argv" argv - std unicode vouch raw_memory
"$Y" build "$W/argv" -o "$W/argv.prog" > "$W/argv.build" 2>&1 || { cat "$W/argv.build"; exit 1; }
python3 -I "$CHECKS/sites.py" "$W/argv.prog.report.md" argv
"$W/argv.prog" "a,b,,c" ","; "$W/argv.prog" "🍎é🍎" "🍎"
"$W/argv.prog" "abc" "" > /dev/null 2>&1 && echo "argv: an empty part GOT THROUGH" || echo "argv: an empty part stops at the call (exit $?)"

# 5. a written empty `at` is refused when building
python3 -I "$CHECKS/programs.py" emptyat | program "$W/emptyat" emptyat - std
"$Y" build "$W/emptyat" -o "$W/emptyat.prog" > "$W/emptyat.build" 2>&1 && echo "emptyat: BUILT (wrong)" || echo "emptyat: $(head -1 "$W/emptyat.build")"

# 6. inventory, unchanged, against std/ (it uses char.%point, so it lists the unicode limb)
program "$W/inv" inv "$R/examples/inventory.cy" std unicode
"$Y" build "$W/inv" -o "$W/inv.prog" > "$W/inv.build" 2>&1 || { cat "$W/inv.build"; exit 1; }
"$W/inv.prog" | diff -q - "$R/examples/inventory.out" > /dev/null && echo "inventory: prints its .out ($(first_line "$W/inv.build"))" \
    || echo "inventory: DIFFERS from its .out"
