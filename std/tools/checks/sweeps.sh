#!/bin/sh
# sweeps.sh: the two wide case sweeps (2026-10-24), each built in the oracle and by kayo against std/.
#   sigma: Σ beside 816 characters (all both cased and case-ignorable, every 15th cased-only, every 10th
#          case-ignorable-only), 3,264 texts, each checked against Table 3-17 computed from Unicode's own files.
#   dots:  the tr/az/lt dot rules with a mark between, 1,906 texts, the oracle's built-in against std.
. "$(dirname "$0")/common.sh"
[ -d "$UCD" ] || { echo "sweeps: needs Unicode 17.0.0's files in $UCD"; exit 1; }
for w in sigma dots; do
    D="$OUT/$w"; rm -rf "$D"; mkdir -p "$D/prog"
    python3 -I "$CHECKS/sweeps.py" gen "$w" "$D" "$UCD" || exit 1
    toml "$D/prog" "$w" std
    (cd "$D/prog" && "$O" kit.cy > ../oracle.out 2>&1)
    "$Y" build "$D/prog" -o "$D/sweep.prog" > "$D/build.txt" 2>&1 || { echo "$w: kayo REFUSED it:"; head -3 "$D/build.txt"; continue; }
    "$D/sweep.prog" > "$D/kayo.out"
    python3 -I "$CHECKS/sweeps.py" check "$w" "$D" "$UCD"
    echo "  $(first_line "$D/build.txt")"
done
