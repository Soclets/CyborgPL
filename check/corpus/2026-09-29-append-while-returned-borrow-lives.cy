# Finding 2026-09-29 (by hand): a borrow handed back by a call, 'r' := first[lend 's'], is
# used after 1000 appends to 's' have moved its storage. CCA's binary reads freed memory
# (prints 1, then an empty line); the oracle prints 1 and 1003. Both should refuse the appends.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
fn put_u ['m': u64] {
    let mut 'buf': std::list[u8] = [];
    let mut 'v': u64 = 'm';
    let mut 'p': u64 = 1;
    while 'v' div.floor 'p' at_least 10 { change 'p' = 'p' times 10; }
    loop {
        std::list.append[lend.mut 'buf', value: to.u8[48 plus ('v' div.floor 'p')]];
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    std::list.append[lend.mut 'buf', value: 10];
    no_check write[1, lend 'buf', std::list.length[lend 'buf']];
}

fn first [lent 'a': std::list[i32]] -> lent std::list[i32] { return lend 'a'; }

MAIN {
    let mut 's': std::list[i32] = [1, 2, 3];
    let 'r' := first[lend 's'];
    for 'k': i32 [from: 1, to: 1000, by: 1] { std::list.append[lend.mut 's', value: 'k']; }
    put_u[to.u64['r'[pos: 1]]];
    put_u[std::list.length[lend 'r']];
}
