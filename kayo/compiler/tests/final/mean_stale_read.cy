# Not an average: one element read once, before the loop, and added on
# every trip — the count is the input's, not the list's length. The sum
# over length template must not fit (its read has to be in the loop, at the
# current position), so the final build refuses the narrowing.
extern.C fn exit ['code': i32];
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn atoi ['s': ptr[u8]] -> i32;
fn stale [lent 'xs': std::list[i32], 'n': i32] -> i32 {
    let mut 'total' := 0;
    let 'v' := lend 'xs';
    let mut 'i' := 1;
    let 'x' := 'v'[pos: 'i'];
    while 'i' at_most 'n' {
        change 'total' = 'total' plus 'x';
        change 'i' = 'i' plus 1;
    }
    return 'total' div.floor std::list.length[lend 'xs'];
}
MAIN {
    let 'k' := "N";
    no_check let 'n': i32 = atoi[getenv[lend 'k']];
    let 'xs': std::list[i32] = [2147483647,];
    no_check my.exit[stale[lend 'xs', 'n'] mod.floor 256];
}
