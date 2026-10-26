#!/bin/sh
# case.sh [UNITS]: str.upper/lower over gen_case.py's edge texts beside the oracle's built-in std, in the oracle and
# through kayo. UNITS defaults to all of them: ascii,simple,full,full.root,full.tr,full.az,full.lt.
. "$(dirname "$0")/common.sh"
U="${1:-ascii,simple,full,full.root,full.tr,full.az,full.lt}"
W="$OUT/case"; rm -rf "$W"; mkdir -p "$W"
stdtest "$W"
python3 -I "$CHECKS/gen_case.py" pair "$U" | program "$W/pair" pair - builtin "stdtest=../kits/std" unicode money
python3 -I "$CHECKS/gen_case.py" real "$U" | program "$W/real" real - std unicode money
(cd "$W/pair" && "$O" . > ../pair.out 2>&1) || { echo "case, in the oracle: FAILED"; head -3 "$W/pair.out"; }
"$Y" build "$W/real" -o "$W/real.prog" > "$W/real.build" 2>&1 || { echo "case: kayo REFUSED it:"; head -4 "$W/real.build"; exit 1; }
"$W/real.prog" > "$W/real.out"
python3 -I -c "
import sys
P = open(sys.argv[1], encoding='utf-8').read().splitlines(); K = open(sys.argv[2], encoding='utf-8').read().splitlines()
def keyed(ls, tag): return dict((l[2:].split(' [', 1)[0], l[2:]) for l in ls if l.startswith(tag + ' '))
A = keyed(P, 'A'); B = keyed(P, 'B'); Ka = keyed(K, 'A')
dB = [k for k in A if B.get(k) != A[k]]; dK = [k for k in Ka if A.get(k) != Ka[k]]
print('case', sys.argv[3] + ':', len(A), 'built-in answers | stdtest in the oracle:', len(dB), 'differ | std through kayo:', len(Ka),
      'answers,', len(dK), 'differ |', open(sys.argv[4]).readline().split(': ', 1)[1][:55].strip())
for k in dB[:8]: print('  built-in', A[k], '| stdtest', B.get(k))
for k in dK[:8]: print('  built-in', A.get(k), '| kayo', Ka[k])
" "$W/pair.out" "$W/real.out" "$U" "$W/real.build"
