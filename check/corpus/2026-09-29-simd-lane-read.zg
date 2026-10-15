# Finding 2026-09-29 (by hand, probing simd before generating it): a lane read gives the wrong value
# in CCA's binary. simd.fill[4] into simd[int32, 4], then 'a'[1] prints garbage ("000"); a
# simd[uint8, 16] of 200 reads back 144 in lane 1; an int64 lane prints nothing. simd.max and
# simd.sum are right. The oracle refuses 'a'[1] ("'a'[pos …] indexes"): pos has been gone outside
# #strict since 2026-09-27, so that is the oracle's to fix.
# check claude generator, seed 1
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;

fn emit ['m': uint64, 'neg': bool] {
    let mut 'buf': list[uint8] = [];
    if 'neg' { list.append[lend.mut 'buf', value 45]; }
    let mut 'v': uint64 = 'm';
    let mut 'ds': list[uint8] = [];
    list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': uint64 = list.length[lend 'ds'];
    while 'k' at_least 1 {
        list.append[lend.mut 'buf', value 'ds'[pos 'k']];
        change 'k' = 'k' minus 1;
    }
    list.append[lend.mut 'buf', value 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': uint64] {
    emit['n', false];
}

fn put_i ['n': int64] {
    if 'n' at_least 0 {
        emit[to.uint64['n'], false];
    } else {
        let 'up': int64 = -('n' plus 1);
        emit[to.uint64['up'] plus 1, true];
    }
}

MAIN {
    let 'a': simd[int32, 4] = simd.fill[value 4];
    put_i[to.int64['a'[pos 1]]];
    put_i[to.int64['a'[pos 4]]];
    let 'b': simd[uint8, 16] = simd.fill[value 200];
    put_u[to.uint64['b'[pos 1]]];
    let 'c': simd[int64, 2] = simd.fill[value -5];
    put_i['c'[pos 2]];
}
