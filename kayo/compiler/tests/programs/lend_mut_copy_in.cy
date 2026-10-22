# A plain value stored into a `lend.mut` walk's item is written into the
# list, and borrows nothing: 'v' stays free to use (check claude, seed
# 10411). (5 + 7) x 2 + the first item, 7 = 31; the oracle agrees.
extern.C fn exit ['code': i32];
MAIN {
    let 'v': u8 = 5;
    let mut 'xs': std::list[u8] = [1, 2];
    let mut 't': i32 = 0;
    for 'x' in lend.mut 'xs' {
        change 'x' = 'v';
        change 'x' = 7;
        change 't' = 't' plus to.i32['v'] plus to.i32['x'];
    }
    no_check my.exit['t' plus to.i32['xs'[pos: 1]]];
}
