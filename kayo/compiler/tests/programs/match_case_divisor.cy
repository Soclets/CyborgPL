# Layer 3 must see every case of a match (check claude, 2026-09-29, seed
# 1990): 'd' shr 35 is 0 for 284, so 5 div 'v' divides by zero. The query
# kept only the first case's value, 1, and "proved" the division; and it
# shifted a u64 arithmetically. Now it traps, as the oracle stops.
%extern.C proc exit [$code: i32];
union.tag E:
    a; b: u64;
;union.tag E
MAIN:
    let $v: u64 := match E.b(284):
        case a:
            is 1;
        ;case a
        case b $d:
            is $d shr 35;
        ;case b
    ;match;
    let $n: u64 := 5 div.floor $v;
    no_check my.exit(to.i32($n mod.floor 100));
;MAIN
