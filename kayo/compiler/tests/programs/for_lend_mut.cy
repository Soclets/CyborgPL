# Over `lend.mut 'xs'`, the loop's element is the list's own: changing it
# changes the list (owner, 2026-09-30: "the program looks legal to me").
# [10, 20, 30] doubled sums to 120; 100000 twos made sixes, in slices,
# sum to 600000, 192 mod 256; (120 + 192) mod 256 is 56.
extern.C fn exit ['code': i32];
MAIN {
    let mut 'prices': list[i32] = [10, 20, 30];
    for 'x' in lend.mut 'prices' {
        change 'x' = 'x' times 2;
    }
    let 'n': i64 = 100000;
    let mut 'xs': list[i64] = list.fill[value: 2, count: 'n'];
    for 'x' in lend.mut 'xs' {
        change 'x' = 'x' times 3;
    }
    let mut 'total': i64 = 0;
    for 'k': i64 [from: 1, to: 3, by: 1] {
        change 'total' = 'total' plus to.i64['prices'[pos: 'k']];
    }
    for 'v' in lend 'xs' {
        change 'total' = 'total' plus 'v';
    }
    no_check my.exit[to.i32['total' mod.floor 256]];
}
