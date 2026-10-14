# A case's data is checked where it lands, at run time too (check claude, seed
# 15174): -5 into a `uint64` case stops, as the oracle does: exit 101.
extern.C fn exit ['code': int32];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
enum E1 {
    c1: uint32,
    c4: uint64
}
MAIN {
    let 's' := "xyz";
    no_check let 'n': uint64 = strlen[lend 's'];
    let 'k': int64 = to.int64['n'] minus 8;
    let 'e' := E1.c4['k'];
    match 'e' {
        c1 'a' { no_check my.exit[1]; }
        c4 'b' { no_check my.exit[to.int32['b' mod.floor 256]]; }
    }
}
