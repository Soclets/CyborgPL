# Finding 2026-09-29 (by hand): 's' is given away while 'r', a lend of it, is still used.
# CCA refuses it ('r' still borrows it and is used after this); the oracle runs it and prints 1.
# Also the oracle: change 's'[1] = 99 while 'r' is used runs and prints 99.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
fn put_u ['m': u64] {
    let mut 'buf': std::list[u8] = [];
    let mut 'v': u64 = 'm';
    let mut 'p': u64 = 1;
    while 'v' div.floor 'p' at_least 10 { change 'p' = 'p' times 10; }
    loop {
        std::list.append(lend.mut 'buf', value: to.u8(48 plus ('v' div.floor 'p')));
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    std::list.append(lend.mut 'buf', value: 10);
    no_check write(1, lend 'buf', std::list.length(lend 'buf'));
}

MAIN {
    let 's': std::list[i32] = [1, 2, 3];
    let 'r' := lend 's';
    let 't' := give 's';
    put_u(to.u64('r'[pos: 1]));
}
