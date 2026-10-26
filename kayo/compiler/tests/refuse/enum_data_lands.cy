# refuse: the value can be outside what a `u64` holds
# A case's data lands where a field's does, and is checked there (check claude,
# seed 15174): -5 into a `u64` case wrapped to 2^64 - 5, unchecked.
%extern.C fn exit [$code: i32];
enum E1 {
    c1: u32,
    c4: u64
}
MAIN {
    let $e := E1.c4(0 minus 5);
    match $e {
        c1 $a { no_check my.exit(1); }
        c4 $b { no_check my.exit(to.i32($b mod.floor 256)); }
    }
}
