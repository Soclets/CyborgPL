# Finding 2026-09-29 (by hand, probing simd before generating it): a lane read gives the wrong value
# in CCA's binary. simd.fill[4] into simd[i32, count: 4], then 'a'[1] prints garbage ("000"); a
# simd[u8, count: 16] of 200 reads back 144 in lane 1; an i64 lane prints nothing. simd.max and
# simd.sum are right. The oracle refuses 'a'[1] ("'a'[pos: …] indexes"): pos has been gone outside
# #strict since 2026-09-27, so that is the oracle's to fix.
# check claude generator, seed 1
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': list[u8] = [];
    if 'neg' { list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': list[u8] = [];
    list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': u64 = list.length[lend 'ds'];
    while 'k' at_least 1 {
        list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
        change 'k' = 'k' minus 1;
    }
    list.append[lend.mut 'buf', value: 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': u64] {
    emit['n', false];
}

fn put_i ['n': i64] {
    if 'n' at_least 0 {
        emit[to.u64['n'], false];
    } else {
        let 'up': i64 = -('n' plus 1);
        emit[to.u64['up'] plus 1, true];
    }
}

MAIN {
    let 'a': simd[i32, count: 4] = simd.fill[value: 4];
    put_i[to.i64['a'[pos: 1]]];
    put_i[to.i64['a'[pos: 4]]];
    let 'b': simd[u8, count: 16] = simd.fill[value: 200];
    put_u[to.u64['b'[pos: 1]]];
    let 'c': simd[i64, count: 2] = simd.fill[value: -5];
    put_i['c'[pos: 2]];
}
