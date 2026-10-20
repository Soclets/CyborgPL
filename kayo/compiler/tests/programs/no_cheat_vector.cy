# A loop under `no_cheat` keeps every round and may still be vectorised (owner,
# 2026-10-01 and 2026-10-18: only skipping work is cheating; design claude,
# 2026-10-20). 1 + … + 100000 = 5000050000, mod 256 = 80; the last round's
# 1000 x 1000 = 1000000, mod 7 = 1: exits 81.
extern.C fn exit ['code': i32];
MAIN {
    let mut 'acc': i64 = 0;
    no_cheat for seq 'i': i64 [from: 1, to: 100_000, by: 1] {
        change 'acc' = 'acc' plus 'i';
    }
    let mut 'last': i64 = 0;
    no_cheat for seq 'i': i64 [from: 1, to: 1000, by: 1] {
        change 'last' = 'i' times 'i';
    }
    no_check my.exit[to.i32[('acc' mod.floor 256) plus ('last' mod.floor 7)]];
}
