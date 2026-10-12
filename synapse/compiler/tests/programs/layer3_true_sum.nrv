# Layer 3 asked `up['n' + 1] + 9_000_000_000_000_000_000` at 64 bits, where
# it wraps into a small number, so the landing in an `int64` read as proven
# and up(98) — past an int64 in truth — ran on with a wrong number. It is
# asked at the true value now: this must stop. 'n' is 97, and up(97) goes
# through up(98).
extern.C fn exit ['code': int32];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
fn up ['n': int64[min 0, max 100]] -> int64 {
    if 'n' at_least 100 { return 1; }
    return up['n' plus 1] plus 9_000_000_000_000_000_000;
}
MAIN {
    let 's' := "ninety-eight bytes of text, counted by strlen at run time so nothing is known while compiling!!!!";
    no_check let 'n': uint64 = strlen[lend 's'];
    let 'k': int64[min 0, max 100] = to.int64['n' mod.floor 101];
    no_check my.exit[to.int32[up['k'] mod.floor 256]];
}
