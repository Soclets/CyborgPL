# Finding 2026-09-29 (by hand): the oracle checks borrows only on the paths it runs. Both
# wrong borrows sit in an if that is not taken (a borrow of a local handed back; an append
# while 'q' is used), so the oracle prints 7 and exits 0. CCA refuses the program.
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
fn b1 [lent 'a': std::list[i32]] -> lent std::list[i32] {
    let 'loc': std::list[i32] = [1, 2];
    return lend 'loc';
}
MAIN {
    put_u[7];
    let 's': std::list[i32] = [5,];
    let 'n': i32 = 3;
    if 'n' > 5 {
        let 'r' := b1[lend 's'];
        put_u[std::list.length[lend 'r']];
    }
    let mut 't': std::list[i32] = [5,];
    if 'n' > 5 {
        let 'q' := lend 't';
        std::list.append[lend.mut 't', value: 1];
        put_u[std::list.length[lend 'q']];
    }
}
