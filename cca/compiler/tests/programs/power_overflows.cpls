# 3 xx 50 is past any 64-bit type: the check in between stops it.
extern.C fn exit ['code': int32];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
MAIN {
    let 's' := "ab";
    no_check let 'two': int64 = to.int64[strlen[lend 's']];
    let 'p': int64 = ('two' plus 1) pow ('two' times 25);
    no_check my.exit[to.int32['p' mod.floor 256]];
}
