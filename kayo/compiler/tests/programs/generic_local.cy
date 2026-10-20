# A type written inside a generic body names the function's blank (2026-10-12,
# corner_shop): `let 'sum': <T> = 0;` was refused as "a blank `<T>` cannot
# appear here". 1 + 2 + 3 is 6, and 40 + 2 is 42: 48.
extern.C fn exit ['code': i32];

interface Number {
    i32
    i64
}

fn total_of <T: Number> [lent 'xs': list[<T>]] -> <T> {
    let 'sum': <T> = 0;
    for 'x' in lend 'xs' {
        change 'sum' = 'sum' plus 'x';
    }
    return 'sum';
}

MAIN {
    let 'a': list[i32] = [1, 2, 3];
    let 'b': list[i64] = [40, 2];
    let 't' := to.i64[total_of[lend 'a']] plus total_of[lend 'b'];
    no_check my.exit[to.i32['t']];
}
