# Finding 2026-09-29 (by hand, probing borrows before generating them): 's' is appended to
# while 'r', a lend of it, is still used. Both engines let it through: the oracle prints 4,
# CCA's binary prints the stale length 3. Rust-style borrowing (core-features.md) refuses it,
# as CCA already does for change 's'[1] = …
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
fn put_u ['m': uint64] {
    let mut 'buf': list[uint8] = [];
    let mut 'v': uint64 = 'm';
    let mut 'p': uint64 = 1;
    while 'v' div.floor 'p' at_least 10 { change 'p' = 'p' times 10; }
    loop {
        list.append[lend.mut 'buf', value to.uint8[48 plus ('v' div.floor 'p')]];
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    list.append[lend.mut 'buf', value 10];
    no_check write[1, lend 'buf', list.length[lend 'buf']];
}

MAIN {
    let mut 's': list[int32] = [1, 2, 3];
    let 'r' := lend 's';
    list.append[lend.mut 's', value 4];
    put_u[list.length[lend 'r']];
}
