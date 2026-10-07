# The position beside the item (owner, 2026-09-29: a): `for 'i', 'x' in
# lend 'xs'` — 'i' is 1, 2, 3 … (a uint64), 'x' the item there; `perm`
# keeps both after the loop (examples/for_position.cpls as an exit code:
# best at 2, and after the loop 3 and 88 → 2 x 100 + 3 x 10 + 8 = 238).
extern.C fn exit ['code': int32];
MAIN {
    let 'scores': list[int32] = [70, 95, 88];
    let mut 'best_at': uint64 = 0;
    let mut 'best': int32 = 0;
    for 'i', 's' in lend 'scores' {
        if 's' > 'best' { change 'best' = 's'; change 'best_at' = 'i'; }
    }
    for perm 'n', 'y' in lend 'scores' { }
    no_check my.exit[to.int32['best_at'] times 100 plus to.int32['n'] times 10 plus 'y' minus 80];
}
