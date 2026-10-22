# Finding 2026-09-29 (by hand): a function hands back the borrow another call handed it,
# return first[lend 'a']. The oracle runs it (prints 2); CCA refuses it (`second` hands back
# a borrow: `return lend 'a';`, of one of its `lent` parameters). 46faa74: a returned borrow
# comes from a lent parameter in its group, and this one does, through first.
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
fn second [lent 'a': std::list[i32]] -> lent std::list[i32] { return first[lend 'a']; }

MAIN {
    let 's': std::list[i32] = [1, 2, 3];
    let 'r' := second[lend 's'];
    put_u[to.u64['r'[pos: 2]]];
}
