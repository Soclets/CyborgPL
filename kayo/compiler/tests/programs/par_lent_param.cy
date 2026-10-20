# Loops in slices over a lent parameter: its slot is the address it borrows
# already, so the slices take a copy of what is there, not of the address.
# The oracle: 64.
extern.C fn exit ['code': i32];
fn total [lent 'xs': list[i64]] -> i64 {
    let mut 'acc': i64 = 0;
    for 'x' in lend 'xs' {
        change 'acc' = 'acc' plus ('x' mod.floor 7);
    }
    return 'acc';
}
fn scale [lent.mut 'xs': list[i64], 'k': i64] {
    for 'i': u64 from 1 to list.length[lend 'xs'] by 1 {
        change 'xs'[pos 'i'] = ('xs'[pos 'i'] times 'k') mod.floor 1000;
    }
}
MAIN {
    let mut 'xs': list[i64] = list.fill[value 3, count 200000];
    scale[lend.mut 'xs', 5];
    let 'a' := total[lend 'xs'];
    no_check my.exit[to.i32['a' mod.floor 256]];
}
