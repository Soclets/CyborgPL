# Finding 2026-09-29 (by hand): b1's result is in @x, and so is 'b' (ownership-syntax.md:
# "everything marked @x lives and dies together"), so giving away 't', lent as 'b', while 'r'
# is used is refused. CCA refuses it at the give; the oracle follows the body (only 'a' is
# returned) and prints 1.
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
fn b1 [@x] [lent @x 'a': std::list[i32], lent @x 'b': std::list[i32]] -> lent @x std::list[i32] {
    return lend 'a';
}
MAIN {
    let 's': std::list[i32] = [5,];
    let 't': std::list[i32] = [6, 7];
    let 'r' := b1(lend 's', lend 't');
    let 'g' := give 't';
    put_u(std::list.length(lend 'r'));
}
