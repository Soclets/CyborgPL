# Finding 2026-10-09 (check run --borrow-wrong 1000, seed 42640, cut down by hand): 's'[pos: 2] shr 2, a
# lane of a simd[u64, count: 2], in a comparison. A lane has the lane type, so the shift has a width (owner,
# 2026-10-07: from the shifted number), as a list item does since 328b04b1. The oracle stops: "`shr`
# needs a width, and nothing here gives one". Synapse runs it and prints 1.
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

fn b1 [@y, @x] [lent @y 'a': list[u16], lent @x 'b': list[u16], lent @x 'c': list[u16]] -> lent @x list[u16] {
    if list.length[lend 'b'] at_least list.length[lend 'c'] {
        return lend 'c';
    }
    return lend 'b';
}

fn b2 [@x] [lent @x 'a': list[u16]] -> lent @x list[u16] {
    return lend 'a';
}

fn b3 [@x, @y] [lent @x 'a': list[u16], lent @y 'b': list[u16], lent @x 'c': list[u16]] -> lent @x list[u16] {
    if list.length[lend 'c'] at_least list.length[lend 'a'] {
        return b2[lend 'a'];
    }
    return lend 'c';
}

fn b4 [@x] [lent 'a': list[u16], lent 'b': list[u16], lent @x 'c': list[u16]] -> lent @x list[u16] {
    return lend 'c';
}

MAIN {
    let 's': simd[u64, count: 2] = simd.fill[value: 40];
    if 3 < ('s'[pos: 2] shr 2) { put_u[1]; } else { put_u[0]; }
}
