# Parallel `for` (owner, 2026-09-30: "do para for"): loops whose iterations
# are proven apart run in slices on the cores — each writes its own element,
# or adds into one total — and give what the loop in order gives. The sum
# is 0 or more every time, so its one check at the end is the loop's.
# (Their total mod 256, as the oracle computes it: 160.)
extern.C fn exit ['code': int32];
MAIN {
    let 'n': int64 = 300000;
    let mut 'xs': list[int64] = list.fill[value 3, count 'n'];
    for 'i': int64 from 1 to 'n' by 1 {
        change 'xs'[pos 'i'] = ('xs'[pos 'i'] times 7) mod.floor 11;
    }
    let mut 'total': int64 = 0;
    for 'x' in lend 'xs' {
        change 'total' = 'total' plus ('x' mod.floor 7);
    }
    let mut 'grid': list[int64] = list.fill[value 0, count 2000];
    for 'r': int64 from 1 to 2000 by 1 {
        let mut 'row': int64 = 0;
        for 'c': int64 from 1 to 200 by 1 {
            change 'row' = 'row' plus (('r' times 'c') mod.floor 5);
        }
        change 'grid'[pos 'r'] = 'row';
    }
    let mut 'g': int64 = 0;
    for 'v' in lend 'grid' {
        change 'g' = 'g' plus 'v';
    }
    no_check my.exit[to.int32[('total' plus 'g') mod.floor 256]];
}
