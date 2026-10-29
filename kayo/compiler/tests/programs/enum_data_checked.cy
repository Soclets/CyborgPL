# A case's data is checked where it lands, at run time too (check claude, seed
# 15174): -5 into a `u64` case stops, as the oracle does: exit 101.
%extern.C proc exit [$code: i32];
%extern.C proc strlen [$s: ptr[u8]] -> u64;
union.tag E1:
    c1: u32;
    c4: u64;
;union.tag E1
MAIN:
    let $s := "xyz";
    no_check let $n: u64 := strlen(lend $s);
    let $k: i64 := to.i64($n) - 8;
    let $e := E1.c4($k);
    match $e:
        case c1 $a:
            no_check my.exit(1);
        ;case c1
        case c4 $b:
            no_check my.exit(to.i32($b mod.floor 256));
        ;case c4
    ;match
;MAIN
