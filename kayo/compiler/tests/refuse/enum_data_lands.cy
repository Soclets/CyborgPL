# refuse: the value can be outside what a `u64` holds
# A case's data lands where a field's does, and is checked there (check claude,
# seed 15174): -5 into a `u64` case wrapped to 2^64 - 5, unchecked.
%extern.C proc exit [$code: i32];
union.tag E1:
    c1: u32;
    c4: u64;
;union.tag E1
MAIN:
    let $e := E1.c4(0 - 5);
    match $e:
        case c1 $a:
            no_check my.exit(1);
        ;case c1
        case c4 $b:
            no_check my.exit(to.i32($b mod.floor 256));
        ;case c4
    ;match
;MAIN
