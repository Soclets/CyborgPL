# `for !cheat` (owner, 2026-10-01): every round really runs, in order with
# `seq` or in slices without it, and gives what the loop in order gives.
# 1 + … + 300000 = 45000150000, mod 256 = 240; 1 + … + 10 = 55; 295 exits
# as 39, as the oracle gives.
extern.C fn exit ['code': i32];
MAIN {
    let mut 'acc': i64 = 0;
    no_cheat for 'i': i64 from 1 to 300_000 by 1 {
        change 'acc' = 'acc' plus 'i';
    }
    let mut 'more': i64 = 0;
    no_cheat for seq 'i': i64 from 1 to 10 by 1 {
        change 'more' = 'more' plus 'i';
    }
    no_check my.exit[to.i32[('acc' mod.floor 256) plus 'more']];
}
